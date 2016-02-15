#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;
extern crate crypto;
extern crate dotenv;
extern crate docopt;
extern crate rustc_serialize;

use std::env;
use std::process;
use std::io::Write;

use docopt::Docopt;

mod util;
mod db;
mod cmd;
mod md5_hasher;
mod file_util;

macro_rules! wout {
  ($($arg:tt)*) => ({
    (writeln!(&mut ::std::io::stdout(), $($arg)*)).unwrap();
  });
}

macro_rules! werr {
  ($($arg:tt)*) => ({
    (writeln!(&mut ::std::io::stderr(), $($arg)*)).unwrap();
  });
}

macro_rules! fail {
  ($e:expr) => (Err(::std::convert::From::from($e)));
}

macro_rules! command_list {
  () => (
"
  help             Show command usage
  store_locations  Recreate images for all store locations
  stores           Recreate images for all stores
  products         Recreate images for all products
  buzz_messages    Recreate images for all buzz messages
"
  )
}

static USAGE: &'static str = concat!("
Usage:
  bolt <command> [<args>...]
  bolt [options]

Options:
  --list        List all commands available.
  -h, --help    Display this message
  --version     Print version info and exit

Commands:", command_list!());

#[derive(RustcDecodable)]
struct Args {
  arg_command: Option<CliCommand>,
  flag_list: bool
}

fn main() {
  let args: Args = Docopt::new(USAGE)
                          .and_then(|d| d.options_first(true)
                                         .version(Some(util::version()))
                                         .decode())
                          .unwrap_or_else(|e| e.exit());

  if args.flag_list {
    wout!(concat!("Installed commands:", command_list!()));
    return;
  }

  match args.arg_command {
    None => {
      werr!(concat!(
        "Please choose one of the following commands:",
        command_list!()));
      process::exit(0);
    }
    Some(cmd) => {
      if cmd.run() {
        process::exit(0);
      } else {
        werr!("Process failed with no error!");
        process::exit(0);
      }
    }
  }
}

#[derive(Debug)]
enum CliCommand {
  Help,
  StoreLocations,
  Stores,
  Products,
  BuzzMessages
}

impl CliCommand {
  fn run(self) -> bool {
    let argv: Vec<_> = env::args().map(|v| v.to_owned()).collect();
    let argv: Vec<_> = argv.iter().map(|s| &**s).collect();
    let argv = &*argv;

    match self {
      CliCommand::Help           => { wout!("{}", USAGE); true }
      CliCommand::StoreLocations => cmd::store_locations::run(argv),
      CliCommand::Stores         => cmd::stores::run(argv),
      CliCommand::Products       => cmd::products::run(argv),
      CliCommand::BuzzMessages   => cmd::buzz_messages::run(argv)
    }
  }
}

impl rustc_serialize::Decodable for CliCommand {
  fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<CliCommand, D::Error> {
    let c = try!(d.read_str());
    Ok(match &*c {
      "help"            => CliCommand::Help,
      "store_locations" => CliCommand::StoreLocations,
      "stores"          => CliCommand::Stores,
      "products"        => CliCommand::Products,
      "buzz_messages"   => CliCommand::BuzzMessages,
      _                 => { let err = concat!("Installed commands:", command_list!()); return Err(d.error(&*err)); }
    })
  }
}
