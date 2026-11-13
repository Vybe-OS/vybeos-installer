use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
pub struct Installer {
    pub disk: String,
    pub locale: String,
    pub timezone: String,
    pub hostname: String,
    pub username: String,
    pub password: String,
}

fn dry_run() -> bool {
    std::env::var("VYBEOS_DRY_RUN").is_ok()
}

pub fn ensure_live_env() -> Result<(), String> {
    if dry_run() || Path::new("/run/archiso").exists() {
        Ok(())
    } else {
        Err("live environment required".into())
    }
}

pub fn sh(
    cmd: &str,
    args: &[&str],
    stdin: Option<&str>,
    on_log: &mut impl FnMut(&str),
) -> Result<(), String> {
    on_log(&format!("$ {cmd} {}", args.join(" ")));
    if dry_run() {
        return Ok(());
    }

    let mut child = Command::new("sh")
        .args(["-c", "exec \"$0\" \"$@\" 2>&1", cmd])
        .args(args)
        .stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{cmd}: {e}"))?;

    if let Some(input) = stdin {
        child.stdin.take().unwrap()
            .write_all(input.as_bytes())
            .map_err(|e| e.to_string())?;
    }

    for line in BufReader::new(child.stdout.take().unwrap()).lines() {
        on_log(&line.map_err(|e| e.to_string())?);
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmd} failed ({status})"))
    }
}

fn part_path(disk: &str, n: u8) -> String {
    if disk.ends_with(|c: char| c.is_ascii_digit()) {
        format!("{disk}p{n}")
    } else {
        format!("{disk}{n}")
    }
}

const PACKAGES: &str = include_str!("packages");
fn parse_packages(src: &str) -> Vec<&str> {
    src.lines()
        .flat_map(|l| l.split('#').next().unwrap_or("").split_whitespace())
        .collect()
}

fn chroot(
    args: &[&str],
    stdin: Option<&str>,
    on_log: &mut impl FnMut(&str),
) -> Result<(), String> {
    let mut a = vec!["/mnt"];
    a.extend(args);
    sh("arch-chroot", &a, stdin, on_log)
}

impl Installer {
    pub fn run(&self, on_log: &mut impl FnMut(&str)) -> Result<(), String> {
        ensure_live_env()?;
        let result = self.steps(on_log);
        let _ = sh("umount", &["-R", "/mnt"], None, on_log); // sukses atau gagal
        result
    }

    fn steps(&self, on_log: &mut impl FnMut(&str)) -> Result<(), String> {
        self.partition(on_log)?;
        self.format_and_mount(on_log)?;
        self.pacstrap(on_log)?;
        self.genfstab(on_log)?;
        self.configure_in_chroot(on_log)?;
        self.install_bootloader(on_log)
    }

    fn partition(&self, on_log: &mut impl FnMut(&str)) -> Result<(), String> {
        let _ = sh("umount", &["-R", "/mnt"], None, on_log);
        let _ = sh("swapoff", &["-a"], None, on_log);

        // deleting existing partition table
        sh("wipefs", &["-a", "-f", &self.disk], None, on_log)?;

        let script = "label: gpt\nsize=1GiB, type=U\ntype=L\n";
        sh(
            "sfdisk",
            &["--wipe", "always", "--wipe-partitions", "always", &self.disk],
            Some(script),
            on_log,
        )?;

        sh("udevadm", &["settle"], None, on_log)?;
        Ok(())
    }

    fn format_and_mount(&self, on_log: &mut impl FnMut(&str)) -> Result<(), String> {
        let esp = part_path(&self.disk, 1);
        let root = part_path(&self.disk, 2);

        // format
        sh("mkfs.fat", &["-F", "32", &esp], None, on_log)?;
        sh("mkfs.btrfs", &["-f", &root], None, on_log)?;

        // creating subvolume
        sh("mount", &[&root, "/mnt"], None, on_log)?;
        for sv in ["@", "@home", "@log", "@pkg"] {
            sh("btrfs", &["subvolume", "create", &format!("/mnt/{sv}")], None, on_log)?;
        }
        sh("umount", &["/mnt"], None, on_log)?;

        // mounting subvolume
        let opt = |sv: &str| format!("subvol={sv},compress=zstd");
        sh("mount", &["-o", &opt("@"), &root, "/mnt"], None, on_log)?;
        sh(
            "mkdir",
            &["-p", "/mnt/boot", "/mnt/home", "/mnt/var/log", "/mnt/var/cache/pacman/pkg"],
            None,
            on_log,
        )?;
        sh("mount", &["-o", &opt("@home"), &root, "/mnt/home"], None, on_log)?;
        sh("mount", &["-o", &opt("@log"), &root, "/mnt/var/log"], None, on_log)?;
        sh("mount", &["-o", &opt("@pkg"), &root, "/mnt/var/cache/pacman/pkg"], None, on_log)?;

        sh("mount", &["-o", "umask=0077", &esp, "/mnt/boot"], None, on_log)?;
        Ok(())
    }

    fn pacstrap(&self, on_log: &mut impl FnMut(&str)) -> Result<(), String> {
        let mut args: Vec<&str> = vec!["-K", "/mnt"];
        args.extend(parse_packages(PACKAGES));
        sh("pacstrap", &args, None, on_log)
    }

    fn genfstab(&self, on_log: &mut impl FnMut(&str)) -> Result<(), String> {
        sh(
            "sh",
            &["-c", "genfstab -U /mnt > /mnt/etc/fstab"],
            None,
            on_log,
        )
    }

    fn configure_in_chroot(&self, on_log: &mut impl FnMut(&str)) -> Result<(), String> {
        self.config_time(on_log)?;
        self.config_locale(on_log)?;
        self.config_hostname(on_log)?;
        self.config_user(on_log)?;
        self.config_system(on_log)
    }

    fn config_time(&self, on_log: &mut impl FnMut(&str)) -> Result<(), String> {
        let zone = format!("/usr/share/zoneinfo/{}", self.timezone);
        chroot(&["ln", "-sf", &zone, "/etc/localtime"], None, on_log)?;
        chroot(&["hwclock", "--systohc"], None, on_log)
    }

    fn config_locale(&self, on_log: &mut impl FnMut(&str)) -> Result<(), String> {
        let charset = self
            .locale
            .split_once('.')
            .map(|(_, c)| c)
            .unwrap_or("UTF-8");

        let mut locale_gen = String::from("en_US.UTF-8 UTF-8\n");
        if self.locale != "en_US.UTF-8" {
            locale_gen.push_str(&format!("{} {}\n", self.locale, charset));
        }

        sh("tee", &["/mnt/etc/locale.gen"], Some(&locale_gen), on_log)?;
        chroot(&["locale-gen"], None, on_log)?;
        sh(
            "tee",
            &["/mnt/etc/locale.conf"],
            Some(&format!("LANG={}\n", self.locale)),
            on_log,
        )
    }

    fn config_hostname(&self, on_log: &mut impl FnMut(&str)) -> Result<(), String> {
        sh("tee", &["/mnt/etc/hostname"], Some(&format!("{}\n", self.hostname)), on_log)
    }

    fn config_user(&self, on_log: &mut impl FnMut(&str)) -> Result<(), String> {
        chroot(
            &["useradd", "-m", "-G", "wheel,docker", "-s", "/bin/bash", &self.username],
            None,
            on_log,
        )?;

        chroot(
            &["chpasswd"],
            Some(&format!("{}:{}\n", self.username, self.password)),
            on_log,
        )?;

        sh(
            "tee",
            &["/mnt/etc/sudoers.d/10-wheel"],
            Some("%wheel ALL=(ALL:ALL) ALL\n"),
            on_log,
        )?;
        sh("chmod", &["440", "/mnt/etc/sudoers.d/10-wheel"], None, on_log)?;

        chroot(&["passwd", "-l", "root"], None, on_log)
    }

    fn config_system(&self, on_log: &mut impl FnMut(&str)) -> Result<(), String> {
        sh(
            "tee",
            &["/mnt/etc/systemd/zram-generator.conf"],
            Some("[zram0]\nzram-size = min(ram / 2, 4096)\ncompression-algorithm = zstd\n"),
            on_log,
        )?;

        sh(
            "tee",
            &["/mnt/etc/greetd/config.toml"],
            Some("[terminal]\nvt = 1\n\n[default_session]\ncommand = \"agreety --cmd sway\"\nuser = \"greeter\"\n"),
            on_log,
        )?;

        chroot(
            &["systemctl", "enable", "greetd", "NetworkManager", "docker", "systemd-timesyncd"],
            None,
            on_log,
        )
    }

    fn install_bootloader(&self, on_log: &mut impl FnMut(&str)) -> Result<(), String> {
        let root = part_path(&self.disk, 2);

        // 1. create cmdline file for UKI
        sh(
            "sh",
            &[
                "-c",
                r#"mkdir -p /mnt/etc/kernel && echo "root=UUID=$(blkid -s UUID -o value "$1") rootflags=subvol=@ rw zswap.enabled=0" > /mnt/etc/kernel/cmdline"#,
                "_",
                &root,
            ],
            None,
            on_log,
        )?;

        // 2. buat direktori EFI/Linux di ESP, supaya bisa dipasang UKI
        sh("mkdir", &["-p", "/mnt/boot/EFI/Linux"], None, on_log)?;
        let preset = r#"ALL_config="/etc/mkinitcpio.conf"
            ALL_kver="/boot/vmlinuz-linux"

            PRESETS=('default' 'fallback')

            default_uki="/boot/EFI/Linux/vybeos.efi"

            fallback_uki="/boot/EFI/Linux/vybeos-fallback.efi"
            fallback_options="-S autodetect"
            "#;
        sh("tee", &["/mnt/etc/mkinitcpio.d/linux.preset"], Some(preset), on_log)?;

        // 3. buang initramfs sisa pacstrap (dibuat pakai preset lama)
        sh("sh", &["-c", "rm -f /mnt/boot/initramfs-linux*.img"], None, on_log)?;

        // 4. build UKI
        chroot(&["mkinitcpio", "-P"], None, on_log)?;

        // 5. pasang systemd-boot ke ESP (/boot)
        chroot(&["bootctl", "install"], None, on_log)?;

        // 6. loader.conf; UKI di EFI/Linux otomatis ketemu, gak perlu entri manual
        sh(
            "tee",
            &["/mnt/boot/loader/loader.conf"],
            Some("timeout 3\nconsole-mode max\neditor no\n"),
            on_log,
        )
    }
}