use rust_on_rails::prelude::*;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::str::FromStr;

use serde_json::Value; // for getting bitcoin price from coinbase.com

const SATS: u64 = 1_000_000;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct WalletKey(Option<Xpriv>);

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct MemoryPersister(ChangeSet);
impl WalletPersister for MemoryPersister {
    type Error = ();
    fn initialize(persister: &mut Self) -> Result<ChangeSet, Self::Error> {Ok(persister.0.clone())}
    fn persist(persister: &mut Self, changeset: &ChangeSet) -> Result<(), Self::Error> {Ok(persister.0.merge(changeset.clone()))}
}

pub struct CachePersister(Arc<Mutex<MemoryPersister>>);
#[async_trait]
impl Task for CachePersister {
    fn interval(&self) -> Option<Duration> {Some(Duration::from_secs(1))}

    async fn run(&mut self, h_ctx: &mut HeadlessContext) {
        // println!("persisting");
        let mut change_set = h_ctx.cache.get::<MemoryPersister>().await;
        {
            let mut amcs = self.0.lock().unwrap();
            amcs.0.merge(change_set.0);
            change_set = (*amcs).clone();
        }
        // println!("chang_set: {:?}", change_set.0.indexer);
        h_ctx.cache.set(&change_set).await;
    }
}

pub struct BDKPlugin {
    wallet: PersistedWallet<MemoryPersister>,
    persister: Arc<Mutex<MemoryPersister>>,
    recipient_address: Arc<Mutex<Option<Address>>>,
    price: Arc<Mutex<f32>>,
}
impl BDKPlugin {
    pub async fn init(&mut self) {//Include theme
        // println!("Initialized BDK");
    }

    pub async fn get_descriptors(cache: &mut Cache) -> (Bip86<Xpriv>, Bip86<Xpriv>) {
        //TODO: get key from ios keychain
        let key = cache.get::<WalletKey>().await.0.unwrap_or(
            Xpriv::new_master(Network::Bitcoin, &secp256k1::SecretKey::new(&mut secp256k1::rand::thread_rng()).secret_bytes()).unwrap()
        );
        // println!("key: {}", hex::encode(key.private_key.secret_bytes()));
        cache.set(&WalletKey(Some(key.clone()))).await;
        (Bip86(key.clone(), KeychainKind::External), 
         Bip86(key, KeychainKind::Internal))
    }

    pub async fn get_wallet(cache: &mut Cache) -> (PersistedWallet<MemoryPersister>, MemoryPersister) {
        let mut db = cache.get::<MemoryPersister>().await; 
        // println!("db_index: {:?}", db.0.indexer);
        let (ext, int) = Self::get_descriptors(cache).await;
        let network = Network::Bitcoin;
        let wallet_opt = Wallet::load()
            .descriptor(KeychainKind::External, Some(ext.clone()))
            .descriptor(KeychainKind::Internal, Some(int.clone()))
            .extract_keys()
            .check_network(network)
            .load_wallet(&mut db)
            .expect("wallet");
        (match wallet_opt {
            Some(wallet) => wallet,
            None => {
                // println!("created: {:?}", db.0.indexer);
                Wallet::create(ext, int)
                .network(network)
                .create_wallet(&mut db)
                .expect("wallet")
            }
        }, db)
    }

    pub fn get_balance(&self) -> Amount {
        self.wallet.balance().total()
    }

    pub fn get_new_address(&mut self) -> Address {
        let address = self.wallet.reveal_next_address(KeychainKind::External);
        // println!("a: {:?}", address);
        self.wallet.persist(&mut self.persister.lock().unwrap()).expect("write is okay");
        address.address
    }

    pub fn get_price(&self) -> f32 {
        *self.price.lock().unwrap()
    }

    pub fn set_recipient_address(&mut self, address: String) -> bool {
        if let Some(add) = Address::from_str(&address).ok() {
            *self.recipient_address.lock().unwrap() = add.require_network(Network::Bitcoin).ok();
            true
        } else { false }
    }

    pub fn get_recipient_address(&self) -> Option<Address> {
        self.recipient_address.lock().unwrap().clone()
    }

    pub fn get_fees(&mut self, btc: f64) -> (f32, f32) {
        // println!("RUNNING GET FEES");
        let address = self.get_recipient_address().expect("Address not found.");
        // println!("GOT ADDRESS");
        let mut tx_builder = self.wallet.build_tx();
        // println!("BUILD TX BUILDER");
        tx_builder
            .add_recipient(address.script_pubkey(), Amount::from_btc(btc).expect("Could not parse"))
            .fee_rate(FeeRate::from_sat_per_vb(5).expect("valid feerate"));
        // println!("BEFORE");
        let psbt = tx_builder.finish().expect("HUh?");
        // println!("AFTER: psdbt {:?}", psbt);
        (0.0, 0.0)
    }

    pub fn get_dust_limit(&self) -> f32 {(546 / SATS) as f32}
    

    // pub fn get_transactions(&self) -> Vec<WalletTx<'_>> {
    //     self.wallet.transactions_sort_by(|tx1, tx2| {
    //         tx1.chain_position.cmp(&tx2.chain_position)
    //     })
    // }

    // get bitcoin price
    // set recipient address
    // get recipient address
    // set send amount
    // get send amount
    // set priority
    // get priority
    // get_fee (priority) -> (f32, f32)
    // build transaction
    // submit transaction
    // get all transactions

}

impl Plugin for BDKPlugin {
    async fn background_tasks(ctx: &mut HeadlessContext) -> Tasks {
        let (wallet, persister) = Self::get_wallet(&mut ctx.cache).await;
        tasks![WalletSync(wallet, persister)]
    }

    async fn new(ctx: &mut Context, h_ctx: &mut HeadlessContext) -> (Self, Tasks) {
        let (wallet, persister) = Self::get_wallet(&mut h_ctx.cache).await;
        let persister = Arc::new(Mutex::new(persister));
        let price = Arc::new(Mutex::new(0.0));
        (BDKPlugin{
            wallet, 
            persister: persister.clone(), 
            price: price.clone(), 
            recipient_address: Arc::new(Mutex::new(None)),
        }, tasks![CachePersister(persister), GetPrice(price)])
    }
}

pub struct WalletSync(PersistedWallet<MemoryPersister>, MemoryPersister);
#[async_trait]
impl Task for WalletSync {
    fn interval(&self) -> Option<Duration> {Some(Duration::from_secs(5))}

    async fn run(&mut self, h_ctx: &mut HeadlessContext) {
        let mut sync_request = self.0.start_sync_with_revealed_spks()
        .inspect(|item, sync| {println!("items: {:?}, script: {:?}", item, sync);})
        .build();


        let mut scan_request = self.0.start_full_scan().build();

        let builder = Builder::new("https://blockstream.info/api");
        let blocking_client = builder.build_blocking();
        //let res = blocking_client.sync(sync_request, 1).unwrap();
        let res = blocking_client.full_scan(scan_request, 10, 1).unwrap();
        // println!("res: {:?}", res);
        self.0.persist(&mut self.1).expect("write is okay");
        let change_set = h_ctx.cache.get::<MemoryPersister>().await.0;
        self.1.0.merge(change_set);
        h_ctx.cache.set(&self.1.0).await;
    }
}

