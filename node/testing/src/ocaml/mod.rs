use std::{
    env,
    fs::{self, File},
    io,
    os::unix::prelude::PermissionsExt,
    process::{Child, Command},
};

use libp2p::Multiaddr;

pub struct Node {
    child: Child,
    pub port: u16,
    pub peer_id: libp2p::PeerId,
}

impl Node {
    pub fn spawn_berkeley(port: u16, rest_port: u16, peers: Option<&[&Multiaddr]>) -> Self {
        fs::remove_dir_all("/root/.mina-config").unwrap_or_default();
        let id = rand::random::<u64>();
        let temp_dir = env::temp_dir().join(format!("mina-test-key-{id:016x}"));
        fs::create_dir_all(&temp_dir).expect("create test dir");
        fs::set_permissions(&temp_dir, PermissionsExt::from_mode(0o700)).expect("access metadata");
        let temp_key = temp_dir.join("key");
        Command::new("mina")
            .env("MINA_LIBP2P_PASS", "")
            .args(&["libp2p", "generate-keypair", "--privkey-path"])
            .arg(&temp_key)
            .output()
            .expect("generate key");
        let peer_id = temp_dir.join("key.peerid");
        let peer_id =
            io::read_to_string(File::open(peer_id).expect("peed id file")).expect("peer_id");
        let peer_id = peer_id
            .trim_end_matches('\n')
            .parse()
            .expect("peer id is invalid");
        let mut cmd = Command::new("mina");

        cmd.env("MINA_LIBP2P_PASS", "")
            .env("DUNE_PROFILE", "devnet")
            .args(&[
                "daemon",
                "--libp2p-keypair",
                temp_key.display().to_string().as_str(),
                "--insecure-rest-server",
                "--external-port",
                port.to_string().as_str(),
                "--rest-port",
                rest_port.to_string().as_str(),
            ]);
        if let Some(peers) = peers {
            cmd.args(
                peers
                    .into_iter()
                    .map(|p| ["--peer".to_string(), p.to_string()])
                    .flatten(),
            );
        } else {
            cmd.args(&[
                "--peer-list-url",
                "https://storage.googleapis.com/seed-lists/berkeley_seeds.txt",
            ]);
        }

        let child = cmd.spawn().expect("ocaml node");

        Self {
            child,
            port,
            peer_id,
        }
    }

    pub fn local_addr(&self) -> libp2p::Multiaddr {
        format!("/ip4/127.0.0.1/tcp/{}/p2p/{}", self.port, self.peer_id)
            .parse()
            .expect("must be valid")
    }

    pub fn kill(&mut self) {
        self.child.kill().expect("kill");
    }
}

#[cfg(test)]
#[test]
fn run_ocaml() {
    use std::io::{BufRead, BufReader};

    let mut node = Node::spawn_berkeley(8302, 3086, None);
    let stdout = node.child.stdout.take().unwrap();
    std::thread::spawn(move || {
        for line in BufRead::lines(BufReader::new(stdout)) {
            println!("{}", line.unwrap());
        }
    });

    node.child.wait().unwrap();
    node.kill();
}
