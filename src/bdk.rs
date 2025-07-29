use pelican_ui::{Context, Plugin};
use bitcoin::BDKTransaction;

use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use std::str::FromStr;

use bdk_wallet::{Wallet, KeychainKind, ChangeSet, Update, LoadParams};
use bdk_wallet::descriptor::template::Bip86;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::bitcoin::{Amount, Network, Address, Txid, FeeRate};
use bdk_wallet::{PersistedWallet, WalletPersister};
use bdk_wallet::chain::{Merge, ChainPosition, Anchor};
use bdk_esplora::esplora_client::Builder;
use bdk_esplora::EsploraExt;

use chrono::{Local, Utc, NaiveDateTime, TimeZone};
use serde_json::Value;
use reqwest::Client;

const SATS: f32 = 100_000_000.0;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct WalletKey(Option<Xpriv>);

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct MemoryPersister(ChangeSet);
impl WalletPersister for MemoryPersister {
    type Error = ();
    fn initialize(persister: &mut Self) -> Result<ChangeSet, Self::Error> {Ok(persister.0.clone())}
    fn persist(persister: &mut Self, changeset: &ChangeSet) -> Result<(), Self::Error> {persister.0.merge(changeset.clone()); Ok(())}
}

pub struct CachePersister(Arc<Mutex<MemoryPersister>>);
#[async_trait]
impl Task for CachePersister {
    fn interval(&self) -> Option<Duration> {Some(Duration::from_secs(1))}

    async fn run(&mut self, h_ctx: &mut HeadlessContext) {
        println!("persisting");
        let mut persister = h_ctx.cache.get::<MemoryPersister>().await;
        {
            let mut amcs = self.0.lock().unwrap();
            amcs.0.merge(persister.0);
            persister = (*amcs).clone();
        }
        h_ctx.cache.set(&persister).await;
    }
}

pub struct GetPrice(Arc<Mutex<f32>>);
#[async_trait]
impl Task for GetPrice {
    fn interval(&self) -> Option<Duration> {Some(Duration::from_secs(10))}

    async fn run(&mut self, _h_ctx: &mut HeadlessContext) {
        println!("Getting price");
        let url = "https://api.coinbase.com/v2/prices/spot?currency=USD";
        let body = reqwest::get(url).await.expect("Could not get url").text().await.expect("Could not get text");
        let json: Value = serde_json::from_str(&body).expect("Could not serde to string");
        let price_str = json["data"]["amount"].as_str().expect("Could not serde to str");
        let price: f32 = price_str.parse().expect("Could not parse as f32");
        *self.0.lock().unwrap() = price;
    }
}

pub struct BDKPlugin {
    persister: Arc<Mutex<MemoryPersister>>,
    wallet: Arc<Mutex<Option<PersistedWallet<MemoryPersister>>>>,
    transactions: Arc<Mutex<Vec<BDKTransaction>>>,
    price: Arc<Mutex<f32>>,
}

impl BDKPlugin {
    pub async fn _init(&mut self) {//Include theme
        println!("Initialized BDK");
    }

    pub async fn get_descriptors(cache: &mut Cache) -> (Bip86<Xpriv>, Bip86<Xpriv>) {
        //TODO: from web 5 normally
        let key = cache.get::<WalletKey>().await.0.unwrap_or(
            Xpriv::new_master(Network::Bitcoin, &secp256k1::SecretKey::new(&mut secp256k1::rand::thread_rng()).secret_bytes()).unwrap()
        );
        println!("key: {}", hex::encode(key.private_key.secret_bytes()));
        cache.set(&WalletKey(Some(key))).await;
        (Bip86(key, KeychainKind::External), 
         Bip86(key, KeychainKind::Internal))
    }

    pub fn get_wallet(&self) -> MutexGuard<Option<PersistedWallet<MemoryPersister>>> {
        // println!("Getting wallet");
        let mut wallet_guard = self.wallet.lock().unwrap();
    
        if wallet_guard.is_none() {
            println!("Wallet was none");
            let mut db = self.persister.lock().unwrap();
            let wallet = PersistedWallet::load(&mut *db, LoadParams::new())
                .expect("Could not load wallet")
                .expect("Wallet was none.");
            *wallet_guard = Some(wallet);
        }
    
        wallet_guard
    }    
    pub async fn create_wallet(cache: &mut Cache) -> MemoryPersister {
        let mut db = cache.get::<MemoryPersister>().await; 
        let (ext, int) = Self::get_descriptors(cache).await;
        let network = Network::Bitcoin;
        let wallet_opt = Wallet::load()
            .descriptor(KeychainKind::External, Some(ext.clone()))
            .descriptor(KeychainKind::Internal, Some(int.clone()))
            .extract_keys()
            .check_network(network)
            .load_wallet(&mut db)
            .expect("wallet");
        match wallet_opt {
            Some(wallet) => wallet,
            None => {
                println!("created: {:?}", db.0.indexer);
                Wallet::create(ext, int)
                .network(network)
                .create_wallet(&mut db)
                .expect("wallet")
            }
        };
        db
    }

    pub fn find_transaction(&mut self, txid: Txid) -> Option<BDKTransaction> {
        self.transactions.lock().unwrap().iter().find(|tx| tx.txid == txid).cloned()
    }

    pub fn get_transactions(&mut self) -> Vec<BDKTransaction> {
        self.transactions.lock().unwrap().to_vec()
    }
    
    pub fn get_balance(&mut self) -> Amount {
        let mut wallet = self.get_wallet();
        wallet.as_mut().unwrap().balance().total()
    }

    pub fn get_new_address(&mut self) -> Address {
        let mut persister = self.persister.lock().unwrap();
        let mut wallet = self.get_wallet();
        let wallet = wallet.as_mut().unwrap();
        let address = wallet.reveal_next_address(KeychainKind::External);
        wallet.persist(&mut persister).expect("write is okay");
        address.address
    }

    pub fn get_price(&self) -> f32 {
        *self.price.lock().unwrap()
    }

    pub fn estimate_fees(&mut self, sats: Amount, address: Address) -> (Amount, Amount) {
        println!("SATS: {:?}", sats);
        let mut wallet = self.get_wallet();
        let wallet = wallet.as_mut().unwrap();
    
        let standard_rate = FeeRate::from_sat_per_vb(3).unwrap();
        let priority_rate = FeeRate::from_sat_per_vb(5).unwrap();
    
        let mut builder_std = wallet.build_tx();
        builder_std.add_recipient(address.script_pubkey(), sats);
        builder_std.fee_rate(standard_rate);
        let psbt_std = builder_std.finish().unwrap();

        let tx_std = psbt_std.clone().extract_tx().unwrap();
        let tx_size_std = tx_std.weight().to_vbytes_ceil();
        let standard_fee = tx_size_std * standard_rate.to_sat_per_vb_floor();

        let mut builder_prio = wallet.build_tx();
        builder_prio.add_recipient(address.script_pubkey(), sats);
        builder_prio.fee_rate(priority_rate);
        let psbt_prio = builder_prio.finish().unwrap();

        let tx_prio = psbt_prio.clone().extract_tx().unwrap();
        let tx_size_prio = tx_prio.weight().to_vbytes_ceil();
        let priority_fee = tx_size_prio * priority_rate.to_sat_per_vb_floor();
    
        (Amount::from_sat(standard_fee), Amount::from_sat(priority_fee))
    }
    
    pub fn get_dust_limit(&self) -> f32 { 0.000005462 }

    pub fn broadcast_transaction(&self, address: Address, amount: Amount, fee_rate: FeeRate) {
        let mut wallet = self.get_wallet();
        let wallet = wallet.as_mut().unwrap();

        let mut builder = wallet.build_tx();
        builder.add_recipient(address.script_pubkey(), amount).fee_rate(fee_rate);

        let mut psbt = builder.finish().expect("Could not build transaction");

        wallet.sign(&mut psbt, bdk_wallet::SignOptions::default()).expect("could not sign transaction");

        println!("PSBT: {:?}", psbt);

        let finalized_tx = psbt.extract_tx().unwrap();

        println!("FINALIZED TX: {:?}", finalized_tx);

        let client = Builder::new("https://blockstream.info/api").build_blocking();
        client.broadcast(&finalized_tx).expect("ERROR BROADCASTING");

        // Ok(tx_details)
    }

    // build transaction
    // submit transaction
}

impl Plugin for BDKPlugin {
    async fn background_tasks(ctx: &mut HeadlessContext) -> Tasks {
        let persister = Self::create_wallet(&mut ctx.cache).await;
        tasks![WalletSync(persister)]
    }

    async fn new(_ctx: &mut Context, h_ctx: &mut HeadlessContext) -> (Self, Tasks) {
        let persister = Self::create_wallet(&mut h_ctx.cache).await;
        let persister = Arc::new(Mutex::new(persister));
        let price = Arc::new(Mutex::new(0.0));
        let transactions = Arc::new(Mutex::new(Vec::new()));

        (BDKPlugin{
            persister: persister.clone(), 
            price: price.clone(), 
            wallet: Arc::new(Mutex::new(None)),
            transactions: transactions.clone(),
        }, tasks![CachePersister(persister), GetPrice(price), GetTransactions(transactions)])
    }
}

pub struct WalletSync(MemoryPersister);
#[async_trait]
impl Task for WalletSync {
    fn interval(&self) -> Option<Duration> {Some(Duration::from_secs(5))}

    async fn run(&mut self, h_ctx: &mut HeadlessContext) {
        let mut persister = h_ctx.cache.get::<MemoryPersister>().await;
        let mut wallet = PersistedWallet::load(&mut persister, LoadParams::new()).expect("Could not load wallet").expect("Wallet was none.");

        let scan_request = wallet.start_full_scan().build();

        let builder = Builder::new("https://blockstream.info/api");
        let blocking_client = builder.build_blocking();
        let res = blocking_client.full_scan(scan_request, 10, 1).unwrap();
        wallet.apply_update(Update::from(res)).unwrap();

        wallet.persist(&mut persister).expect("write is okay");
        let persister2 = h_ctx.cache.get::<MemoryPersister>().await;
        self.0.0.merge(persister2.0);
        h_ctx.cache.set(&persister).await;
    }
}

pub struct GetTransactions(Arc<Mutex<Vec<BDKTransaction>>>);
#[async_trait]
impl Task for GetTransactions {
    fn interval(&self) -> Option<Duration> {Some(Duration::from_secs(10))}

    async fn run(&mut self, h_ctx: &mut HeadlessContext) {
        let mut persister = h_ctx.cache.get::<MemoryPersister>().await;
        let wallet = PersistedWallet::load(&mut persister, LoadParams::new()).expect("Could not load wallet").expect("Wallet was none.");
        let mut transactions = Vec::new();

        for canonical_tx in wallet.transactions_sort_by(|tx1, tx2| tx2.chain_position.cmp(&tx1.chain_position)).into_iter().map(|tx| tx.clone()) {
            let tx_node = &canonical_tx.tx_node;
            let tx = &tx_node.tx;

            let (datetime, btc_price) = match &canonical_tx.chain_position {
                ChainPosition::Confirmed { anchor, .. } => {
                    let unix_timestamp = get_block_time(anchor.anchor_block().hash.to_string()).await.unwrap_or(0);
                    let utc = Utc.timestamp_opt(unix_timestamp as i64, 0).unwrap();
                    let btc_price = get_btc_price_at(utc.format("%Y-%m-%d %H:%M:%S").to_string()).await.unwrap_or(0.0);
                    (Some(utc.with_timezone(&Local)), btc_price)
                }
                _ => (None, 0.0)
            };
            
            let received: u64 = tx.output.iter().filter(|out| wallet.is_mine(out.script_pubkey.clone())).map(|out| out.value.to_sat()).sum();
            let input_sum: u64 = tx.input.iter().filter_map(|input| wallet.get_utxo(input.previous_output)).map(|utxo| utxo.txout.value.to_sat()).sum();
            let sent = input_sum.saturating_sub(received);
            let fee = (input_sum > 0).then(|| input_sum.saturating_sub(tx.output.iter().map(|o| o.value.to_sat()).sum())); // this has to be wrong...
            let is_received = received > sent;

            let address = match is_received {
                true => { 
                    tx.output.iter().find_map(|out| {
                        wallet.is_mine(out.script_pubkey.clone())
                            .then(|| Address::from_script(&out.script_pubkey, wallet.network()).ok()).flatten()
                    })                    
                },
                false => {
                    tx.input.iter().find_map(|input| {
                        wallet.get_utxo(input.previous_output).and_then(|utxo| {
                            Address::from_script(&utxo.txout.script_pubkey, wallet.network()).ok()
                        })
                    })
                }
            };

            transactions.push(BDKTransaction {
                txid: tx_node.txid,
                datetime,
                is_received,
                amount: Amount::from_sat(received.max(sent)),
                price: btc_price,
                fee: fee.map(Amount::from_sat),
                address: address.map(|a| a.to_string()),
            });
        }

        *self.0.lock().unwrap() = transactions;
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SendAddress(Option<String>);

impl SendAddress {
    pub fn new(new: String) -> Self {
        let address = Address::from_str(&new).ok()
            .and_then(|a| a.require_network(Network::Bitcoin).ok())
            .map(|a| a.to_string());

        SendAddress(address)
    }

    pub fn is_valid(&self) -> bool { self.0.is_some() }

    pub fn get(&self) -> &Option<String> { &self.0 }

    pub fn as_address(&self) -> Address { // this cause you can't store address because it doesn't implement deserialize
        Address::from_str(self.0.as_ref().unwrap()).unwrap().require_network(Network::Bitcoin).unwrap()
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SendAmount(Amount); // btc

impl SendAmount {
    pub fn new(new: f64) -> Self {
        SendAmount(Amount::from_sat((new as f32 * SATS).round() as u64))
    }

    pub fn get(&self) -> &Amount { &self.0 }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SendFee(Amount, Amount, bool); // min, max, is_priority

impl SendFee {
    pub fn new(standard: Amount, priority: Amount, is_priority: bool) -> Self {
        SendFee(standard, priority, is_priority)
    }

    pub fn get_fee(&self) -> &Amount { 
        match self.2 {
            false => &self.0,
            true => &self.1
        }
    }

    pub fn as_rate(&self) -> FeeRate { 
        match self.2 {
            false => FeeRate::from_sat_per_vb(3).unwrap(),
            true => FeeRate::from_sat_per_vb(5).unwrap()
        }
    }

    pub fn priority(&mut self) -> &mut bool { &mut self.2 }
    pub fn priority_fee(&self) -> &Amount { &self.0 }
    pub fn standard_fee(&self) -> &Amount { &self.1 }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct CurrentTransaction(Option<BDKTransaction>);

impl CurrentTransaction {
    pub fn new(new: BDKTransaction) -> Self {
        CurrentTransaction(Some(new))
    }

    pub fn get(&self) -> Option<BDKTransaction> { self.0.clone() }
}

async fn get_block_time(block_hash: String) -> Result<u64, Box<dyn std::error::Error>> {
    let url = format!("https://mempool.space/api/block/{}", block_hash);
    let res: Value = Client::new().get(&url).send().await?.json().await?;
    Ok(res["timestamp"].as_u64().ok_or("Missing or invalid timestamp field")?)
}

async fn get_btc_price_at(timestamp: String) -> Result<f64, Box<dyn std::error::Error>> {
    let datetime = NaiveDateTime::parse_from_str(&timestamp, "%Y-%m-%d %H:%M:%S")?;
    let target_timestamp = Utc.from_utc_datetime(&datetime).timestamp();

    let from = target_timestamp - 300;
    let to = target_timestamp + 300;

    let url = format!("https://api.coingecko.com/api/v3/coins/bitcoin/market_chart/range?vs_currency=usd&from={from}&to={to}");
    let val: Value = reqwest::get(&url).await?.json().await?;
    let prices = val["prices"].as_array().ok_or("Missing 'prices' array in response")?;

    let closest_price = prices
        .iter()
        .filter_map(|entry| {
            let arr = entry.as_array()?;
            let ts = arr[0].as_f64()? as i64 / 1000;
            let price = arr[1].as_f64()?;
            Some((ts, price))
        })
        .min_by_key(|(ts, _)| (ts - target_timestamp).abs())
        .map(|(_, price)| price)
        .ok_or("No price data found in the specified range")?;

    Ok(closest_price)
}

pub fn parse_btc_uri(input: String) -> (String, Option<f64>) {
    let mut parts = input.strip_prefix("bitcoin:").unwrap_or(&input).splitn(2, '?');
    let address = parts.next().unwrap_or("");

    let amount = parts.next().and_then(|q| {
        q.split('&').find_map(|kv| {
            let (key, val) = kv.split_once('=')?;
            match key == "amount" {
                true => val.parse::<f64>().ok(),
                false => None
            }
        })
    });

    (address.to_string(), amount)
}


// pub fn _add_password(&self, password: String) {
//     use objc2_core_foundation::CFDictionary;
//     use objc2_core_foundation::CFString;
//     use objc2_core_foundation::CFData;
//     use objc2_core_foundation::CFType;
//     // use objc2_security::SecItemAdd;
//     use objc2_security::SecItemDelete;
//     use objc2_security::kSecAttrService;
//     use objc2_security::kSecValueData;
//     use objc2_security::kSecClassGenericPassword;
//     use objc2_security::kSecAttrAccount;
//     use objc2_security::kSecClass;

//     use std::ptr;
//     let account_cf = CFString::from_str("did::nym::happyduckyforeverrrrrr");
//     let service_cf = CFString::from_str("orange.me");
//     let password_data = CFData::from_bytes(password.as_bytes());
//     let keys: [&CFType; 4] = unsafe { [
//         kSecClass.as_ref(),
//         kSecAttrAccount.as_ref(),
//         kSecAttrService.as_ref(),
//         kSecValueData.as_ref(),
//     ]};

//     let values: [&CFType; 4] = unsafe { [
//         kSecClassGenericPassword.as_ref(),
//         account_cf.as_ref(),
//         service_cf.as_ref(),
//         password_data.as_ref(),
//     ] };

//     let key_callbacks = ptr::null();
//     let value_callbacks = ptr::null();

//     let query = unsafe {
//         CFDictionary::new(
//             None,
//             keys.as_ptr() as *mut *const std::ffi::c_void,
//             values.as_ptr() as *mut *const std::ffi::c_void,
//             keys.len() as isize,
//             key_callbacks,
//             value_callbacks,
//         )
//     };

//     if let Some(query_dict) = query {
//         unsafe { SecItemDelete(query_dict.as_ref()) };
//     } else {
//         println!("Failed to create CFDictionary.");
//     }
// }
