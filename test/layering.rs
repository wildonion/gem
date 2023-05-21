

use crate::*;


/*
    
    ┏———————————————┓
       NFT LAYRING
    ┗———————————————┛

    🍟 A MULTITHREADED AND ASYNC NFT LAYERING TOOLS
    🚧 ------- leetcode algos -------  

*/
pub async fn layering(){

    pub const WIDTH: usize = 32;
    pub const HEIGHT: usize = 32;
    pub const RGBA: usize = 4;

    pub struct Image{
        /*
            there must be HEIGHT number of [u8; RGBA]
            which is like 32 rows X 4 cols and  
            WIDTH number of [[u8; RGBA]; HEIGHT]
            which is like 32 X 32 X 4
        */
        pub hat: [[[u8; RGBA]; HEIGHT]; WIDTH], //// 32 X 32 X 4 => 32 Pixels and RGBA channels
        pub mask: [[[u8; RGBA]; HEIGHT]; WIDTH] //// 32 X 32 X 4 => 32 Pixels and RGBA channels
    }

    let (sender, receiver) = tokio::sync::mpsc::channel::<HashMap<&str, Vec<&str>>>(1024);

    let assets_path = "assets"; //// in the root of the project
    let nfts_path = "nfts"; //// in the root of the project

    fn update_asset_to_path<'s>(
        mut asset_to_path: HashMap<&'s str, Vec<String>>, 
        key: &'s str, key_images: Vec<String>) 
        -> HashMap<&'s str, Vec<String>>{
        asset_to_path.entry(key).and_modify(|v| *v = key_images);
        asset_to_path
    } 

    tokio::spawn(async move{

        // hashmap can be a 3d arr also and reading 
        // from it is slower that arr and vec 

        let assets_names = &["Beard", "Hat", "Mask"];
        let mut asset_to_path: HashMap<&str, Vec<String>> = HashMap::new(); //// a map of between asset name and their images path
        for asset in assets_names{
            asset_to_path.entry(asset).or_insert(vec![]);
        }

        let assets = std::fs::read_dir(assets_path).unwrap();
        for asset in assets{
            //// since unwrap() takes the ownership of the type 
            //// we've borrowed the asset using as_ref() method
            //// which returns a borrow of the asset object which
            //// let us to have the asset later in other scopes.
            let filename = asset.as_ref().unwrap().file_name();
            let filepath = asset.as_ref().unwrap().path();
            let filepath_string = filepath.display().to_string();
            let mut asset_to_path_clone = asset_to_path.clone();
            let asset_to_path_keys = asset_to_path_clone.keys();
            let filepath_string_clone = filepath_string.clone();
            for key in asset_to_path_keys{ 
                if filepath_string_clone.starts_with(*key){
                    //// if a type is behind an immutable shared reference 
                    //// it can't mutate the data unless we define it's 
                    //// pointer as mutable in the first place or convert 
                    //// it to an owned type which returns Self. 
                    let mut key_images = asset_to_path.get(key).unwrap().to_owned();
                    key_images.push(filepath_string_clone.clone());
                    asset_to_path = update_asset_to_path(asset_to_path.clone(), key, key_images);
                }
            }
        }


        let (sender_flag, mut receiver_flag) = 
        tokio::sync::mpsc::channel::<u8>(1024); //// mpsc means multiple thread can read the data but only one of them can mutate it at a time
        tokio::spawn(async move{

            type Job<T> = std::thread::JoinHandle<T>; 
            let job: Job<_> = std::thread::spawn(||{});
            
            std::thread::scope(|s|{
                s.spawn(|| async{ //// making the closure body as async to solve async task inside of it 
                    sender_flag.send(1).await.unwrap(); //// sending data to the downside of the tokio jobq channel
                    for asset_path in asset_to_path.values(){
                        tokio::spawn(async move{
                            // reading the shared sate data from the
                            // receiver_flag mpsc receiver to acquire 
                            // the lock on the mutexed data.
                            // ... 
                            // make a combo of each asset path in a separate thread asyncly 
                            // while idx < combos.len()!{
                            //     bin(i%3!).await;
                            //     010
                            //     01
                            // }
                            // ...
                        });
                    }
                });
                s.spawn(|| async{
                    sender_flag.send(2).await.unwrap();
                });
                s.spawn(|| async{ //// making the closure body as async to solve async task inside of it 
                    while let Some(input) = receiver_flag.recv().await{ //// waiting on data stream to receive them asyncly
                        // do whatever with the collected data of all workers 
                        // ...
                    }
                    let data: Vec<u8> = receiver_flag.try_recv().into_iter().take(2).collect();
                });
            });
        });
    });

}