extern crate nix;
use crate::{extra, format, PATH_TO_BATTERY_PERCENTAGE, PATH_TO_BATTERY_STATUS};
use nix::unistd;
use std::fs;
use std::process::{Command, Stdio};

/// Read battery percentage from __/sys/class/power_supply/BAT0/capacity__
pub fn battery_percentage() -> String {
    let percentage = fs::read_to_string(PATH_TO_BATTERY_PERCENTAGE);

    let ret = match percentage {
        Ok(ret) => ret,
        Err(_e) => return String::new(),
    };

    extra::pop_newline(ret)
}

/// Read name of the computer as specified in /sys/class/dmi/id/product_version
pub fn product_name() -> String {
    let name = fs::read_to_string("/sys/class/dmi/id/product_version");
    let ret = match name {
        Ok(ret) => ret,
        Err(_e) => return String::from("Could not obtain product name"),
    };
    extra::pop_newline(ret)
}

/// Read battery status from __/sys/class/power_supply/BAT0/status__
pub fn battery_status() -> String {
    let status = fs::read_to_string(PATH_TO_BATTERY_STATUS);
    let ret = match status {
        Ok(ret) => ret,
        Err(_e) => return String::new(),
    };
    extra::pop_newline(ret)
}

/// Read current terminal instance using __ps__ command
pub fn terminal() -> String {
    //  ps -p $$ -o ppid=
    //  $$ doesn't work natively in rust but its value can be
    //  accessed through nix::unistd::getppid()
    let ppid = Command::new("ps")
        .arg("-p")
        .arg(unistd::getppid().to_string())
        .arg("-o")
        .arg("ppid=")
        .output()
        .expect("Failed to get current terminal instance PPID using 'ps -p <PID> o ppid='");

    let terminal_ppid = String::from_utf8(ppid.stdout)
        .expect("'ps' process stdout was not valid UTF-8")
        .trim()
        .to_string();

    let name = Command::new("ps")
        .arg("-p")
        .arg(terminal_ppid)
        .arg("o")
        .arg("comm=")
        .output()
        .expect("Failed to get current terminal instance name using 'ps -p <PID> o comm='");

    String::from_utf8(name.stdout)
        .expect("'ps' process stdout was not valid UTF-8")
        .trim()
        .to_string()
}

/// Read current shell instance name using __ps__ command
pub fn shell(shorthand: bool) -> String {
    //  ps -p $$ -o comm=
    //  $$ doesn't work natively in rust but its value can be
    //  accessed through nix::unistd::getppid()
    if shorthand {
        let output = Command::new("ps")
            .arg("-p")
            .arg(unistd::getppid().to_string())
            .arg("o")
            .arg("comm=")
            .output()
            .expect("Failed to get current shell instance name 'ps -p <PID> o args='");

        let shell_name = String::from_utf8(output.stdout)
            .expect("read_terminal: stdout to string conversion failed");
        return shell_name.trim().to_string();
    }

    // If shell shorthand is false, we use "args=" instead of "comm="
    // to print the full path of the current shell instance name
    let output = Command::new("ps")
        .arg("-p")
        .arg(unistd::getppid().to_string())
        .arg("o")
        .arg("args=")
        .output()
        .expect("Failed to get current shell instance name 'ps -p <PID> o args='");

    let shell_name = String::from_utf8(output.stdout)
        .expect("read_terminal: stdout to string conversion failed");
    String::from(shell_name.trim())
}

/// Extract package count by running /usr/bin/pacman -Qq
pub fn package_count() -> usize {
    let wh = Command::new("which")
        .arg("pacman")
        .output()
        .expect("Failed to start 'which' process");

    let which = String::from_utf8(wh.stdout).expect("'which' process stdout was not valid UTF-8");

    if which.trim() == "/usr/bin/pacman" {
        let pacman = Command::new("pacman")
            .arg("-Q")
            .arg("-q")
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to start 'pacman' process");

        let pac_out =
            String::from_utf8(pacman.stdout).expect("'pacman' process stdout was not valid UTF-8");
        let packages: Vec<&str> = pac_out.split('\n').collect();

        return packages.len() - 1;
    }
    return 0;
}

/// Read kernel version by running "uname -r"
pub fn kernel_version() -> String {
    let output = fs::read_to_string("/proc/version");
    let ret = match output {
        Ok(ret) => ret.split_whitespace().nth(2).unwrap().to_string(),
        Err(_e) => return String::from("Could not obtain kernel version"),
    };
    ret
}

/// Read hostname using __unistd::gethostname()__
pub fn hostname() -> String {
    let output = fs::read_to_string("/etc/hostname");
    let ret = match output {
        Ok(ret) => extra::pop_newline(ret),
        Err(_e) => return String::from("Could not obtain hostname"),
    };
    ret
}

/// Read operating system name from __/etc/os-release__
pub fn operating_system() -> String {
    let mut os =
        String::from(extra::get_line_at("/etc/os-release", 0, "Could not obtain distribution name").unwrap());
    if !os.contains("NAME=\"") {
        return os.replace("NAME=", "");
    }
    os.pop();
    os.replace("NAME=\"", "")
}

/// Read processor information from __/proc/cpuinfo__
pub fn cpu_model_name() -> String {
    let mut cpu = String::from(
        extra::get_line_at("/proc/cpuinfo", 4, "Could not obtain processor model name").unwrap(),
    );

    cpu = cpu
        .replace("model name", "")
        .replace(":", "")
        .trim()
        .to_string();
    cpu
}

/// Read first float (uptime) from __/proc/uptime
pub fn uptime() -> String {
    let uptime = fs::read_to_string("/proc/uptime");
    let ret = match uptime {
        Ok(ret) => format::uptime(ret.split_whitespace().next().unwrap().to_string()),
        Err(_e) => return String::from("Could not obtain uptime"),
    };
    ret
}
