import themis from 'wasm-themis'
import { setupSecureCell } from './secure-cell.js'

window.onload = function() {
    themis.initialized.then(function() {
        const loaded = document.getElementById('wasm-loaded')
        loaded.textContent = 'WasmThemis loaded!'
        loaded.classList.add('dimmed')

        /* ----------------------------------- */
        /* ----------------------------------- */
        const private_key_hex = "524543320000002dec0be77c00f525f94908910f32df7b2f1cafd8888b968c787b5e0a3fe96fcc42e98641127e";
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

        const message = new TextEncoder().encode("1691377675");
        const signedMessage = signer.sign(message);
        const signedMessageHex = bytesToHex(signedMessage);

        console.log(signedMessageHex);
        /* ----------------------------------- */
        /* ----------------------------------- */

        // setupSecureCell()
    })
}
