# RWLock
Questo è un semplice modulo di test per verificare il funzionamento e le performance dei bindings tra Rust e C. <br />
Per realizzare questo bindings è stata seguita la [patch](https://github.com/Rust-for-Linux/linux/commit/d4d791d4aac041fde6eeba0a8f9201d728b52373) del kernel in cui è stato aggiunto il supporto alle workqueue e la presentazione di Linux Foundation presente in pdf nel repository. <br />

## How to install
Gli step necessari sono i seguenti:

1) Aggiungere a rust/bindings/binding_helpers l'header di rwlock.
2) Dichiarare in rust/kernel/sync/lock.rs il nuovo modulo rwlock.
3) Aggiungere "pub use lock::rwlock::{new_rwlock, RwLock};" in rust/kernel/sync.rs
4) Definire il modulo in questione contenente le astrazioni verso le funzioni d'interesse. <br />
**N.B** Per le macro che non sono delle semplici define è necessario creare un helper all'interno di rust/helpers.c. <br /> 
È presente anche la versione corretta di questo file.
5) È necessario compilare nuovamente il kernel: 
    ```bash
        make LLVM=1 -j$(nproc)
    ```
    Dopo installarlo e aggiornare il boot loader:
    ```bash
        sudo make install
        sudo update-grub
    ```