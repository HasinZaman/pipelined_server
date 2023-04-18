mod get_file {

    use std::{
        fs::{self},
        ops::Range,
        path::Path,
        sync::mpsc,
        thread::{self},
    };

    use rand::Rng;

    use crate::{test_tools::file_env::FileEnv};

    enum Command {
        GetFile(String),
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    enum UtilityData {
        GetFile(Result<String, ()>),
    }

    fn get_file(c: Command) -> UtilityData {
        match c {
            Command::GetFile(path) => {
                let path = Path::new(&path);
                if !path.exists() {
                    return UtilityData::GetFile(Err(()));
                }

                match fs::read_to_string(path) {
                    Ok(value) => return UtilityData::GetFile(Ok(value)),
                    Err(_) => return UtilityData::GetFile(Err(())),
                };
            }
        }
    }

    // #[test]
    // fn file_reading_test() {
    //     let (file_names, file_content) = generate_files(100, 100..1000);

    //     assert_eq!(file_names.len(), file_content.len());

    //     let _env_vars: Vec<FileEnv> = file_names
    //         .iter()
    //         .zip(file_content.iter())
    //         .map(|(file_name, content)| FileEnv::new(file_name, content))
    //         .collect();

    //     let utility: UtilityThread<Command, UtilityData> = UtilityThread::new(get_file);

    //     let sender = utility.sender.clone();

    //     file_names
    //         .into_iter()
    //         .enumerate()
    //         .map(|(i1, path)| {
    //             let sender = sender.clone();

    //             thread::spawn(move || {
    //                 let (tx, rx) = mpsc::channel();
    //                 sender
    //                     .send((Command::GetFile(path.to_string()), tx))
    //                     .unwrap();

    //                 let result = rx.recv().unwrap();

    //                 let UtilityData::GetFile(content) = result.clone();

    //                 match content {
    //                     Ok(content) => println!(
    //                         "{i1}\t:\t{path}\t:\t{:?}...",
    //                         content.chars().collect::<Vec<char>>().get(0..5)
    //                     ),
    //                     Err(_) => {}
    //                 };

    //                 result
    //             })
    //         })
    //         .map(|thread| thread.join().unwrap())
    //         .zip(
    //             file_content
    //                 .into_iter()
    //                 .map(|content| UtilityData::GetFile(Ok(content.to_string()))),
    //         )
    //         .for_each(|(actual, expected)| {
    //             assert_eq!(actual, expected);
    //         });

    //     let (tx, rx) = mpsc::channel();

    //     sender
    //         .send((Command::GetFile(String::from("Fake_File.txt")), tx))
    //         .unwrap();

    //     assert_eq!(rx.recv().unwrap(), UtilityData::GetFile(Err(())));
    // }

    fn generate_files(count: usize, file_size: Range<usize>) -> (Vec<String>, Vec<String>) {
        let mut names = Vec::with_capacity(count);
        let mut content = Vec::with_capacity(count);

        for i in 0..count {
            names.push(format!("file_{}.txt", i));

            let size: usize = rand::thread_rng().gen_range(file_size.clone());

            let mut file_content = String::new();

            for _ in 0..size {
                file_content.push(rand::random());
            }

            content.push(file_content);
        }

        (names, content)
    }
}

// mod pooled_thread{
//     use std::{sync::{mpsc, Mutex, Arc}, thread::{JoinHandle, self}};

//     use lazy_static::lazy_static;

//     lazy_static! {
//         static ref THREAD_POOL : Arc<Mutex<Vec<(mpsc::Sender<(u32, mpsc::Sender<u32>)>, JoinHandle<()>)>>> = generate_thread_pool(3);
//     }

//     fn pooled_fib(size: u32) -> u32 {
//         loop {
//             // Find free thread

//             //dfdfdf

//             //dfdfdf

//             //dfdfdf
//             todo!()
//         }
//     }

//     #[test]
//     fn fib_test(){

//     }

//     fn generate_thread_pool(thread_count: usize) -> Arc<Mutex<Vec<(mpsc::Sender<(u32, mpsc::Sender<u32>)>, JoinHandle<()>)>>> {
//         let mut thread_pool = Vec::new();
//         for _ in 0..thread_count {
//             let (tx, rx) = mpsc::channel::<(u32, mpsc::Sender<u32>)>();

//             let t = thread::spawn(
//                 move || {
//                     let (val, tx) = rx.recv().unwrap();

//                     tx.send(fib(val)).unwrap();
//                 }
//             );

//             thread_pool.push((tx, t));
//         }

//         Arc::new(Mutex::new(thread_pool))
//     }

//     fn fib(n: u32) -> u32 {
//         match n {
//             0 => 0,
//             1 => 1,
//             n => fib(n-1) + fib(n-2)
//         }
//     }
// }
