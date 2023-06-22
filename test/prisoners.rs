use rand::Rng; // 0.8.5

fn main() {
    
    use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut rng = thread_rng();
        
        
        for sidx in 0..100{
            let mut boxes = (0..100).collect::<Vec<u8>>();
            boxes.shuffle(&mut rng);
            
            let mut prisoners = (0..100).collect::<Vec<u8>>();
            prisoners.shuffle(&mut rng);
            
            println!("\n->>>>>> sim {sidx:}");

            for pidx in 0..prisoners.len(){
                
                let p_number = prisoners[pidx];
                let mut direction = p_number;
                
                // a prisoner have 50 try max
                for search_idx in 0..50{
                    
                    if search_idx == 49{
                        println!("50 try max reached, no prisoner found his number, all them will be executed");
                    }
                    
                    let selected_box = boxes[direction as usize];
                    if selected_box != p_number{
                        direction = selected_box;
                    } else{
                        println!("prisoner {pidx:} found his number");
                        break;
                    }

                }
            }
        }
}