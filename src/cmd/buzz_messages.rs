use std::env;
use mongodb::db::ThreadedDatabase;
use mongodb::coll::options::WriteModel;
use bson::oid::ObjectId;

use db;
use md5_hasher;
use file_util;

pub fn run(argv: &[&str]) -> bool {
  let connection = db::get_connection();
  let collection_name = "buzz_messages";
  let collection = connection.collection(collection_name);

  let cursor = collection.find(None, None).unwrap();

  println!("Environment - {:?}", argv);
  println!("Retrieving {:?}...", collection_name);

  let source_name = "buzz_message";
  let mut bulk_operations = vec![];

  for result in cursor {
    if let Ok(item) = result {

      let document_id = &* item.get_object_id("_id").unwrap().to_string();

      println!("Starting for document - {:?}", document_id);

      let orig_file_name = &* item.get("image")
                                  .unwrap()
                                  .to_string()
                                  .replace("\"", "");

      let tmp_dir_path    = file_util::create_tmp_dir(&source_name, document_id);
      let source_dir_path = env::var("SOURCE_PATH").unwrap() + "/" + &source_name + "/" + document_id;

      let new_file_name = &* md5_hasher::generate(&*orig_file_name);

      let tmp_file_path    = tmp_dir_path.clone() + "/" + new_file_name;
      let source_file_path = source_dir_path.clone() + "/" + orig_file_name;

      file_util::retrieve_from_source(tmp_file_path, source_file_path);
      file_util::recreate_versions(new_file_name, &*tmp_dir_path);
      file_util::upload_to_source(&*tmp_dir_path, &*source_dir_path);
      file_util::remove_tmp_dir(&*tmp_dir_path);

      let record_id = ObjectId::with_string(document_id).unwrap();

      bulk_operations.push(WriteModel::UpdateOne {
        filter: doc! { "_id"  => record_id },
        update: doc! { "$set" =>  { "image" => new_file_name } },
        upsert: false
      });

      println!("Done for document - {:?}", document_id);
    }
  }

  let result = collection.bulk_write(bulk_operations, true);

  println!("Updated Buzz Messages - {:?}", result.modified_count);

  return true;
}

