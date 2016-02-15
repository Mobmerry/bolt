use std::env;
use std::collections::HashMap;

use mongodb::db::ThreadedDatabase;
use bson::oid::ObjectId;

use db;
use md5_hasher;
use file_util;

pub fn run(argv: &[&str]) -> bool {
  let connection = db::get_connection();
  let collection_name = "stores";
  let collection = connection.collection(collection_name);

  let cursor = collection.find(None, None).unwrap();

  println!("Environment - {:?}", argv);
  println!("Retrieving {:?}...", collection_name);

  let source_name = "store_image";

  for result in cursor {
    if let Ok(item) = result {

      let document_id = &* item.get_object_id("_id").unwrap().to_string();

      println!("Starting for document - {:?}", document_id);

      let mut store_images = HashMap::new();

      let logo_image = item.get_document("logo").unwrap();

      let logo_image_id = logo_image.get_object_id("_id")
                                    .unwrap()
                                    .to_string();

      let logo_image_name = logo_image.get("file")
                                      .unwrap()
                                      .to_string()
                                      .replace("\"", "");

      store_images.insert(logo_image_id, logo_image_name);

      for store_image in item.get_array("images").unwrap() {
        let store_image_json = store_image.to_json();

        let image_id = store_image_json.find("_id")
                                       .unwrap()
                                       .find("$oid")
                                       .unwrap()
                                       .to_string()
                                       .replace("\"", "");

        let image_name = store_image_json.find("file")
                                         .unwrap()
                                         .to_string()
                                         .replace("\"", "");

        store_images.insert(image_id, image_name);
      }

      for (image_id, image_name) in store_images {
        let new_image_name = process(source_name, &*image_id, &*image_name);

        let record_id = ObjectId::with_string(&*image_id).unwrap();

        collection.update_one(
          doc! { "_id" => record_id },
          doc! { "$set" =>  { "logo" => new_image_name } },
          None
        ).expect("Failed to update document.");
      }

      println!("Done for document - {:?}", document_id);
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
