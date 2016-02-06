use crypto::md5::Md5;
use crypto::digest::Digest;

pub fn generate(file_name: &str) -> String {
  let mut hasher = Md5::new();

  hasher.input_str(file_name);

  let new_file_name = hasher.result_str() + ".jpg";

  hasher.reset();

  return new_file_name;
}
