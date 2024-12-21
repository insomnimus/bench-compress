mod byte_size;

use std::{
	fs::File,
	io::{
		self,
		Read,
		Seek,
	},
	path::PathBuf,
	process::{
		Command,
		Stdio,
		exit,
	},
	str::FromStr,
	thread,
	time::Instant,
};

use clap::Parser;
use dur::Duration;

use self::byte_size::ByteSize;

const COMMANDS: &[&str] = &["zstd", "xz", "xz --lzma2=dict=1536Mi,nice=273 -q"];

struct Stats {
	time: Duration,
	after: u64,
	ratio: f64,
}

#[derive(Parser)]
/// Calculate compressability of a file using different programs
struct App {
	// Test for the first N bytes of the file (you can use units such as GiB, MB, KK, KiB etc)
	#[arg(short, default_value_t = ByteSize(5 <<20), value_parser = ByteSize::from_str)]
	n: ByteSize,

	/// Do not print progress messages
	#[arg(short, long)]
	quiet: bool,

	// The file to calculate for
	#[arg()]
	file: PathBuf,

	/// Compression commands to benchmark; each argument is treated as one command, with its arguments separated by whitespace
	#[arg(default_values = COMMANDS)]
	compress_cmd: Vec<String>,
}

fn calculate(cmd: &str, f: &mut File, limit: u64) -> io::Result<Stats> {
	f.seek(io::SeekFrom::Start(0))?;
	let mut c = cmd.split_whitespace();
	let mut child = Command::new(c.next().unwrap())
		.args(c)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.spawn()?;

	let mut stdin = child.stdin.take().unwrap();
	let mut stdout = child.stdout.take().unwrap();

	let thread = thread::spawn(move || {
		let mut total = 0;
		let mut buf = [0; 4096];
		loop {
			let n = stdout.read(&mut buf)?;
			if n == 0 {
				break;
			}
			total += n;
		}

		io::Result::Ok(total as u64)
	});

	let start = Instant::now();

	let write_res = io::copy(&mut f.take(limit), &mut stdin);
	drop(stdin);

	let read_res = thread.join().unwrap();

	let exit_status = child.wait()?;
	if !exit_status.success() {
		return Err(io::Error::new(
			io::ErrorKind::Other,
			format!("process exited with {}", exit_status.code().unwrap_or(-1)),
		));
	}

	let before = write_res?;
	let after = read_res?;

	Ok(Stats {
		time: start.elapsed().into(),
		after,
		ratio: after as f64 / before as f64 * 100.0,
	})
}

fn main() {
	fn run() -> io::Result<()> {
		let args = App::parse();

		let mut f = File::open(&args.file)?;
		let limit = if args.n.0 == 0 {
			u64::MAX
		} else {
			args.n.0 as u64
		};

		let mut results = Vec::with_capacity(args.compress_cmd.len());
		for cmd in args.compress_cmd {
			if !args.quiet {
				eprintln!("measuring {cmd}");
			}

			let res = calculate(&cmd, &mut f, limit)?;
			results.push((cmd, res));
		}

		results.sort_unstable_by(|a, b| b.1.after.cmp(&a.1.after));
		for (cmd, s) in results {
			println!(
				"{cmd}\n{ratio:.2}% = {compressed}; {time}\n",
				ratio = s.ratio,
				compressed = ByteSize(s.after as i64),
				time = s.time
			);
		}

		Ok(())
	}

	if let Err(e) = run() {
		eprintln!("error: {e}");
		exit(1);
	}
}
