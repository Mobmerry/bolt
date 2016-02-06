use std::fs;
use std::process::Command;
use std::thread;
use std::thread::JoinHandle;

pub fn create_tmp_dir(source_name: &str, document_id: &str) -> String {
  let tmp_dir_path = "/tmp/".to_string() + source_name + "-" + document_id;

  match fs::create_dir(&tmp_dir_path) {
    Err(why) => println!("! {:?}", why.kind()),
    Ok(_) => {}
  }

  return tmp_dir_path;
}

pub fn retrieve_from_source(tmp_file_path: String, source_file_path: String) {
  println!("tmp_file_path - {:?}", tmp_file_path);
  println!("source_file_path - {:?}", source_file_path);

  match fs::copy(&source_file_path, &tmp_file_path) {
    Err(why) => println!("Copy file failed! {:?}", why.kind()),
    Ok(_) => {}
  }
}

pub fn recreate_versions(new_file_name: &str, tmp_dir_path: &str) {
  let mut gm_operations = vec![];

  gm_operations.push(resize_image(&new_file_name, &tmp_dir_path, "100x100", "thumb"));
  gm_operations.push(resize_image(&new_file_name, &tmp_dir_path, "225x", "web"));
  gm_operations.push(resize_image(&new_file_name, &tmp_dir_path, "290x", "ldpi"));
  gm_operations.push(resize_image(&new_file_name, &tmp_dir_path, "420x", "mdpi"));
  gm_operations.push(resize_image(&new_file_name, &tmp_dir_path, "520x", "hdpi"));
  gm_operations.push(resize_image(&new_file_name, &tmp_dir_path, "630x", "xhdpi"));
  gm_operations.push(resize_image(&new_file_name, &tmp_dir_path, "1062x", "xxhdpi"));

  for gm_operation in gm_operations {
    // Wait for the thread to finish.
    let _ = gm_operation.join();
  }
}

pub fn upload_to_source(tmp_dir_path: &str, source_dir_path: &str) {
  let file_paths = fs::read_dir(&tmp_dir_path).unwrap();

  clean_source(source_dir_path);

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

pub fn remove_tmp_dir(tmp_dir_path: &str) {
  match fs::remove_dir_all(&tmp_dir_path) {
    Err(why) => println!("Remove dir failed! {:?}", why.kind()),
    Ok(_) => {}
  }
}

fn resize_image(file_name: &str, tmp_dir_path: &str, size: &str, alias: &str) -> JoinHandle<()> {
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
           .arg("-auto-orient")
           .arg("+profile")
           .arg("!icc,!xmp,*")
           .arg(&new_file_name)
           .current_dir(&tmp_dir_path)
           .output()
           .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
  });
}

fn clean_source(source_dir_path: &str) {
  let file_paths = fs::read_dir(&source_dir_path).unwrap();

  for file_path in file_paths {
    match fs::remove_file(file_path.unwrap().path()) {
      Err(why) => println!("Remove file failed! {:?}", why.kind()),
      Ok(_) => {}
    }
  }
}
