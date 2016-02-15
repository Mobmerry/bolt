use std::env;

use mongodb::db::ThreadedDatabase;
use mongodb::coll::options::WriteModel;
use bson::oid::ObjectId;

use db;
use md5_hasher;
use file_util;

pub fn run(argv: &[&str]) -> bool {
  let connection = db::get_connection();
  let collection_name = "products";
  let collection = connection.collection(collection_name);

  let cursor = collection.find(None, None).unwrap();

  println!("Environment - {:?}", argv);
  println!("Retrieving {:?}...", collection_name);

  let source_name = "product_image";

  for result in cursor {
    if let Ok(item) = result {

      let product_id = &* item.get_object_id("_id")
                              .unwrap()
                              .to_string();

      println!("Starting for document - {:?}", product_id);

      let mut bulk_operations = vec![];

      for product_image in item.get_array("images").unwrap() {
        let product_image_json = product_image.to_json();
        let product_oid        = ObjectId::with_string(&*product_id).unwrap();

        let image_id = product_image_json.find("_id")
                                         .unwrap()
                                         .find("$oid")
                                         .unwrap()
                                         .to_string()
                                         .replace("\"", "");

        let image_name = product_image_json.find("file")
                                           .unwrap()
                                           .to_string()
                                           .replace("\"", "");

        let new_image_name = process(source_name, &*image_id, &*image_name);
        let image_oid      = ObjectId::with_string(&*image_id).unwrap();

        bulk_operations.push(WriteModel::UpdateOne {
          filter: doc! { "_id" => product_oid, "images" => { "$elemMatch" => { "_id" => image_oid } } },
          update: doc! { "$set" => { "images.$.file" => new_image_name } },
          upsert: false
        });
      }

      collection.bulk_write(bulk_operations, true);

      println!("Done for document - {:?}", product_id);
    }
  }

  true
}

fn process(source_name: &str, image_id: &str, image_name: &str) -> String {
  let tmp_dir_path    = file_util::create_tmp_dir(&source_name, image_id);
  let source_dir_path = env::var("SOURCE_PATH").unwrap() + "/" + &source_name + "/" + image_id;

  let new_image_name  = &* md5_hasher::generate(&*image_name);

  let tmp_file_path    = tmp_dir_path.clone() + "/" + new_image_name;
  let source_file_path = source_dir_path.clone() + "/" + image_name;

  file_util::retrieve_from_source(tmp_file_path, source_file_path);
  file_util::recreate_versions(new_image_name, &*tmp_dir_path);
  file_util::upload_to_source(&*tmp_dir_path, &*source_dir_path);
  file_util::remove_tmp_dir(&*tmp_dir_path);

  return new_image_name.to_string();
}
