from sshconf import read_ssh_config, empty_ssh_config_file
from os.path import expanduser
from find_ip_pynq import find_ip

new_ip = find_ip()

# print(f"|DEBUG| found ip: {new_ip}")

if new_ip.strip() == "":
    exit(1)

c = read_ssh_config(expanduser("~/.ssh/config"))
c.unset("pz2", "Hostname")
c.set("pz2", Hostname=new_ip)
c.save()