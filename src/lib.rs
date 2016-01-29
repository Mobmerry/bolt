extern crate libc;

use std::ffi::CStr;
use std::fs;
use std::process::Command;
use std::thread;

#[no_mangle]
pub extern fn process(collection_name: *const libc::c_char, collection_id: *const libc::c_char, file_name: *const libc::c_char, new_file_name: *const libc::c_char) {

  let collection_name: &str = unsafe { CStr::from_ptr(collection_name) }.to_str().unwrap();
  let collection_id: &str   = unsafe { CStr::from_ptr(collection_id) }.to_str().unwrap();
  let file_name: &str       = unsafe { CStr::from_ptr(file_name) }.to_str().unwrap();
  let new_file_name: &str   = unsafe { CStr::from_ptr(new_file_name) }.to_str().unwrap();

  let tmp_dir_path = "/tmp/".to_string() + collection_name + "-" + collection_id;

  match fs::create_dir(&tmp_dir_path) {
    Err(why) => println!("! {:?}", why.kind()),
    Ok(_) => {}
  }

  let source_dir_path  = "/home/nitin/work/mobmerry/public/uploads/".to_string() + collection_name + "/" + collection_id;
  let source_file_path = source_dir_path.clone().to_string() + "/" + file_name;

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
