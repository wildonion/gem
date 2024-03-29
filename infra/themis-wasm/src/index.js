import themis from 'wasm-themis'
import { setupSecureCell } from './secure-cell.js'

window.onload = function() {
    themis.initialized.then(function() {
        const loaded = document.getElementById('wasm-loaded')
        loaded.textContent = 'WasmThemis loaded!'
        loaded.classList.add('dimmed')

        /* ----------------------------------- */
        /* ----------------------------------- */
        // public key is: 0x554543320000002d6682f8f7030f89be91e75b5604e14c026d7ec893c4be6de1d221a9e329a59b8dee2fad3b16
        // the verifying process will be done in Rust inside when the message wants to be sent and stored in db
        let hex_pubkey = "0x524543320000002d4fe6311900b579ca7abb58fc8328e2673d1b938681ce696f6a7231a3d87cd5a0a6a08aa997";
        let pubkey = hex_pubkey.slice(2, hex_pubkey.length);
        const private_key_hex = "524543320000002d4fe6311900b579ca7abb58fc8328e2673d1b938681ce696f6a7231a3d87cd5a0a6a08aa997";
        function hexToBytes(hex) {
            const bytes = new Uint8Array(hex.length / 2);
            for (let i = 0; i < hex.length; i += 2) {
                bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
            }
            return bytes;
        }

        function bytesToHex(bytes) {
            let hexArray = [];
            bytes.forEach(byte => {
              hexArray.push(('0' + (byte & 0xFF).toString(16)).slice(-2));
            });
            return hexArray.join('');
          }

        const sms = themis.SecureMessageSign;
        const prv = themis.PrivateKey

        const private_key_bytes = hexToBytes(private_key_hex);
        const signer = new sms(new prv(private_key_bytes));


        let deposit_body = {
            "text": "this is a message sending from chatroom"
        };
        
        // request body signing
        const deposit_message = new TextEncoder().encode(JSON.stringify(deposit_body));
        const signedDepositMessage = signer.sign(deposit_message);
        const signedDepositMessageHex = bytesToHex(signedDepositMessage);

        let final_deposit_sig = "0x"+signedDepositMessageHex;
        deposit_body["tx_sigature"] = final_deposit_sig;

        console.log(final_deposit_sig);
        /* ----------------------------------- */
        /* ----------------------------------- */

        // setupSecureCell()
    })
}
