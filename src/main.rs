#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;
extern crate crypto;
extern crate dotenv;

use std::fs;
use std::env;
use std::process::Command;
use std::thread;

use crypto::md5::Md5;
use crypto::digest::Digest;

use dotenv::dotenv;

use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

use bson::oid::ObjectId;

fn main() {
  println!("Booting BOLT...");

  dotenv().ok();

  let db_host = env::var("MONGO_HOST").unwrap();
  let db_port = 27017; //env::var("MONGO_PORT").unwrap();
  let db_name = env::var("DB_NAME").unwrap();
  let collection_name = env::var("COLLECTION").unwrap();

  println!("Connecting to DB - {:?}:{:?}", db_host, db_port);

  let client = Client::connect(&*db_host, db_port)
                      .ok()
                      .expect("Failed to initialize client.");

  let db = client.db(&*db_name);

  let collection = db.collection(&*collection_name);

  let cursor = collection.find(None, None).unwrap();

  println!("Retrieving documents...");

  for result in cursor {
    if let Ok(item) = result {

      let document_id = item.get_object_id("_id").unwrap().to_string();

      println!("Starting for document - {:?}", document_id);

      let orig_file_name = item.get("logo")
                               .unwrap()
                               .to_string()
                               .replace("\"", "");

      let mut hasher = Md5::new();

      hasher.input_str(&*orig_file_name);

      let new_file_name = hasher.result_str() + ".jpg";

      hasher.reset();

      process(&*document_id, &*orig_file_name, &*new_file_name);

      let record_id = ObjectId::with_string(&*document_id).unwrap();

      collection.update_one(
        doc! { "_id" => record_id },
        doc! { "$set" =>  { "logo" => new_file_name } },
        None
      ).expect("Failed to update document.");

      println!("Done for document - {:?}", document_id);
    }
  }
}

fn process(document_id: &str, orig_file_name: &str, new_file_name: &str) {
  let source_name = env::var("SOURCE").unwrap(); // Singularized name of the collection
  let source_path = env::var("SOURCE_PATH").unwrap(); // path to the public uploads dir

  let tmp_dir_path = "/tmp/".to_string() + &*source_name + "-" + document_id;

  match fs::create_dir(&tmp_dir_path) {
    Err(why) => println!("! {:?}", why.kind()),
    Ok(_) => {}
  }

  let source_dir_path  = source_path + &*source_name + "/" + document_id;
  let source_file_path = source_dir_path.clone().to_string() + "/" + orig_file_name;

  copy_image_to_tmp_dir(&tmp_dir_path, &source_file_path, &new_file_name);

  let mut gm_operations = vec![];

  gm_operations.push(resize_image(&new_file_name, &tmp_dir_path, "100x100", "thumb"));
  gm_operations.push(resize_image(&new_file_name, &tmp_dir_path, "290x", "ldpi"));
  gm_operations.push(resize_image(&new_file_name, &tmp_dir_path, "420x", "mdpi"));
  gm_operations.push(resize_image(&new_file_name, &tmp_dir_path, "520x", "hdpi"));
  gm_operations.push(resize_image(&new_file_name, &tmp_dir_path, "630x", "xhdpi"));
  gm_operations.push(resize_image(&new_file_name, &tmp_dir_path, "1062x", "xxhdpi"));

  for gm_operation in gm_operations {
    // Wait for the thread to finish. Returns a result.
    let _ = gm_operation.join();
  }

  clear_source_dir(&source_dir_path);
  copy_images_to_source_dir(&source_dir_path, &tmp_dir_path);
  remove_tmp_dir(&tmp_dir_path);
}

fn resize_image(file_name: &str, tmp_dir_path: &str, size: &str, alias: &str) -> std::thread::JoinHandle<()> {
  let file_name     = file_name.clone().to_string();
  let new_file_name = (alias.to_string() + "_" + &file_name).clone().to_string();
  let tmp_dir_path  = tmp_dir_path.clone().to_string();
  let size          = size.clone().to_string();

  return thread::spawn(move || {
    Command::new("gm")
           .arg("convert")
           .arg(&file_name)
           .arg("-resize")
           .arg(&size)
           .arg("+profile")
           .arg("!icc,!xmp,*")
           .arg(&new_file_name)
           .current_dir(&tmp_dir_path)
           .output()
           .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
  });
}

fn clear_source_dir(source_dir_path: &str) {
  let file_paths = fs::read_dir(&source_dir_path).unwrap();

  for file_path in file_paths {
    match fs::remove_file(file_path.unwrap().path()) {
      Err(why) => println!("Remove file failed! {:?}", why.kind()),
      Ok(_) => {}
    }
  }
}

fn copy_image_to_tmp_dir(tmp_dir_path: &str, source_file_path: &str, new_file_name: &str) {
  let tmp_file_path = tmp_dir_path.to_string() + "/" + new_file_name;

  match fs::copy(&source_file_path, &tmp_file_path) {
    Err(why) => println!("Copy file failed! {:?}", why.kind()),
    Ok(_) => {}
  }
}

fn copy_images_to_source_dir(source_dir_path: &str, tmp_dir_path: &str) {
  let file_paths = fs::read_dir(&tmp_dir_path).unwrap();

  for file_path in file_paths {
    let unwrapped_file      = file_path.unwrap();
    let unwrapped_file_name = unwrapped_file.file_name();
    let unwrapped_file_path = unwrapped_file.path();

    let file_name = unwrapped_file_name.to_str().unwrap();
    let source_file_path = source_dir_path.to_string() + "/" + file_name;

    match fs::copy(unwrapped_file_path, &source_file_path) {
      Err(why) => println!("Copy file failed! {:?}", why.kind()),
      Ok(_) => {}
    }
  }
}

fn remove_tmp_dir(tmp_dir_path: &str) {
  match fs::remove_dir_all(&tmp_dir_path) {
    Err(why) => println!("Remove dir failed! {:?}", why.kind()),
    Ok(_) => {}
  }
}
