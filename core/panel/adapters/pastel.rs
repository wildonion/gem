


use crate::*;



#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct PastelSenseUploadResponse{
    pub expires_in: String,
    pub image_id: String,
    pub required_preburn_amount: i32,
    pub total_estimated_fee: i32
}


/* ------------------------------------------------------------- */
/*     pastel sense near-duplicate image detection api calls     */
/* ------------------------------------------------------------- */
// https://docs.pastel.network/sense-protocol/building-with-sense-api
pub mod sense{

    pub async fn upload() -> (String, u8){
        
        /* http calls to walletnoe then rpc to supernode

            curl -X 'POST' \
                'http://localhost:8080/openapi/sense/upload' \
                -H 'accept: application/json' \
                -H 'Content-Type: multipart/form-data' \
                -F 'file=@deadkings.jpeg;type=image/jpeg' \
                -F 'filename='
            {"image_id":"wvmREoWK","expires_in":"2023-11-21T08:15:44Z","total_estimated_fee":6175,"required_preburn_amount":1233}
        
        */

        let image_id = String::from("");
        
        (image_id, 0)

    }

    pub async fn start(image_id: &str) -> (String, u8){

        /* 
            curl -X 'POST' \
                'http://localhost:8080/openapi/sense/start/wvmREoWK' \
                -H 'accept: application/json' \
                -H 'Content-Type: application/json' \
                -d '{
                    "app_pastelid": "jXZdb6xo4KsTQCQRngFvtCGvVCaeQJxZEGaAvMktrRd1BTbNAXHcqwPjL3hQy4NiGQDP3HL15LH3GPxy2zMx8b",
                    "burn_txid": "576e7b824634a488a2f0baacf5a53b237d883029f205df25b300b87c8877ab58",
                    "collection_act_txid": "576e7b824634a488a2f0baacf5a53b237d883029f205df25b300b87c8877ab58",
                    "open_api_group_id": "Aliquam quia ullam officia nihil repudiandae."
                }'
        */

        let task_id = String::from("");
        
        (task_id, 0)

    }

}