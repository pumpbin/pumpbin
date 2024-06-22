# 🎃 PumpBin

<p align="center">
  <img src="logo/pumpbin-256.png" height="30%" width="30%">
</p>

**PumpBin** is an Implant Generation Platform.

Modern cybersecurity teams are divided into offensive personnel and cybersecurity researchers, with researchers responsible for producing digital weapons. The teams typically deploy post-exploitation tools like Cobalt Strike, BRC4, or similar.
To evade security software, researchers usually write shellcode loaders, including evasion code to create the final implant. This process generally follows two methods.

1. Offensive personnel provide the shellcode to researchers, who then directly produce the final implant. This method is highly inflexible as offensive personnel must contact researchers every time they need a final implant.

2. Researchers create a binary implant template and provide a final implant generation program. Offensive personnel use this program to inject shellcode into the binary implant template, producing the final implant.

The second method is the reason for the creation of PumpBin, a final implant generation program. Cybersecurity researchers only need to follow PumpBin's guidelines to write implant templates and distribute them along with PumpBin to offensive personnel. (There are very few guidelines as PumpBin is highly flexible.)
![](https://github.com/pumpbin/pumpbin/assets/120295547/549bbfa8-d8a4-44c6-89e1-3f24ef7897d2)

## ✨ Features

* Powerful, simple, and comfortable UI
* Following the minimal principle to ensure maximum flexibility in usage
* Supports two types: shellcode embedded in implants(Local) and hosted on remote servers(Remote)
* Re-selecting the current plugin will generate a new random encryption password
* Filling with random data to ensure each generated implant is unique
* We have user manual, you no longer need to educate your users
* No dependencies, just PumpBin
* Support description, you can write down anything you want to remind users, which is important
* ... And I'm a pumpkin, I have magic🪄
