# Linked List
Questo è un semplice modulo di test per verificare il funzionamento e le performance dei bindings tra Rust e C. <br />
Per realizzare questo bindings è stata seguita la [patch](https://github.com/Rust-for-Linux/linux/commit/d4d791d4aac041fde6eeba0a8f9201d728b52373) del kernel in cui è stato aggiunto il supporto alle workqueue e la presentazione di Linux Foundation presente in pdf nel repository. <br />

## How to install
Gli step necessari sono i seguenti:

1) Aggiungere a rust/bindings/binding_helpers l'header di list.
2) Dichiarare in rust/kernel/lib.rs il nuovo modulo linked_list.
3) Definire il modulo in questione contenente le astrazioni verso le funzioni d'interesse. <br />
**N.B** Per le macro che non sono delle semplici define è necessario creare un helper all'interno di rust/helpers.c. <br /> 
È presente anche la versione corretta di questo file.
4) È possibile generare la documentazione del codice rust relativa al modulo eseguendo all'interno della main directory: 
    ```bash
        make LLVM=1 -j$(nproc) rustdoc
    ``` 
    Questo permette anche di verificare se la sintassi del codice scritto è corretta.

5) È necessario compilare nuovamente il kernel: 
    ```bash
        make LLVM=1 -j$(nproc)
    ```
    Dopo installarlo e aggiornare il boot loader:
    ```bash
        sudo make install
        sudo update-grub
    ```

## Obiettivo
Il modulo è stato scritto sia Rust che in C, in modo tale da verificare le differenze di performance. <br />
Vengono eseguite svariate operazioni scorrendo tutta la lista tenendo traccia del tempo d'esecuzione per poi confrontarlo.