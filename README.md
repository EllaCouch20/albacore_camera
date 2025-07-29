# Web GPU

Before building for Android or IOS you will need to first remove the bin from the cargo.toml and/or the main.rs file from the project tree.

## Android

prerequisites
-Rust toolchain
-Android NDK & SDK installed and configured in PATH

To build for android you must first have `cargo-apk` installed on your system. 

```cargo install cargo-apk```

Once you have installed cargo-apk, then determine the permissions your app requires and specify them in the `Cargo.toml` under the Cargo APK config section.
By default, this wrapper already enables internet permission.

After configuring the proper permissions you can build the apk with...

```cargo apk build```

By default, your build output will appear in `webgpu/target/debug/apk` . Here, you can verify that your permissions were set correctly by viewing the AndroidManifest.xml in the output directory.

## IOS

prerequisites
-Rust Toolchain
-Xcode

To build for IOS you must first have `cargo-lipo` installed on your system.

```cargo install cargo-lipo```

Next you must install `cbindgen`.

```cargo install cbindgen```

Add IOS targets to rustup

```rustup target add aarch64-apple-ios x86_64-apple-ios```

(Optional) Add simulator targets to rustup if desired

```rustup target add aarch64-apple-ios-sim x86_64-apple-ios```

Ensure the `Cargo.toml` specifies the a lib section (this has been provided by default)

```[lib]```
```name = "library_name"```
```crate-type = ["cdylib"] # or "staticlib" if you want a static library rather than dynamic```

Build your library with cargo lipo (specify a release build with the proper flag: `--release`)

```cargo lipo```

This will compile your rust library into `library_name.a` output at `target/universal/debug` or `target/universal/release`

Create a C header using the library name you have specificed in the lib section of the `Cargo.toml`

```cbindgen --config cbindgen.toml --output include/<library_name>.h```

Create an Xcode project in which you wish to use your Rust Library

Copy the dynamic or static library into your Xcode's project library folder

Add the library to your target's "Link Binary With Libraries" in the build phases

Include your C header and ensure Xcode can find it

Make sure to link any dependencies required for your library or framework with Xcode project

Update the "Header Search Paths" to include the directory where your C headers are located

Update "Library Search Paths" to include where your .a file is located

Build and run your app on an IOS device or IOS simulator

## IOS Updated

cd ios

make clean

make run