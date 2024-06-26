<div align="center">
  <a href="https://github.com/pumpbin/pumpbin/releases/latest">
    <img alt="GitHub Release" src="https://img.shields.io/github/v/release/pumpbin/pumpbin?sort=semver&filter=v*.*.*&display_name=tag&style=for-the-badge&labelColor=%2324273a&color=%238aadf4"></a>
    
  <a href="https://github.com/pumpbin/pumpbin/stargazers">
  <img alt="GitHub Repo stars" src="https://img.shields.io/github/stars/pumpbin/pumpbin?style=for-the-badge&labelColor=%2324273a&color=%23f5bde6"></a>
  
  <a href="https://github.com/pumpbin/pumpbin/issues">
  <img alt="GitHub Issues or Pull Requests" src="https://img.shields.io/github/issues/pumpbin/pumpbin?style=for-the-badge&labelColor=%2324273a&color=%23ed8796"></a>

  <a href="https://github.com/pumpbin/pumpbin/blob/main/LICENSE">
    <img alt="GitHub License" src="https://img.shields.io/github/license/pumpbin/pumpbin?style=for-the-badge&labelColor=%2324273a&color=%23eed49f"></a>
  
  <a href="https://x.com/b1nhack">
  <img alt="X (formerly Twitter) Follow" src="https://img.shields.io/twitter/follow/b1nhack?style=for-the-badge&logo=x&label=FOLLOW&labelColor=%2324273a&color=%237dc4e4"></a>
</div>

# 🎃 PumpBin

<p align="center">
  <img src="logo/pumpbin-256x256.png" height="30%" width="30%">
</p>

**PumpBin** is an Implant Generation Platform.

To use PumpBin, you need to have a b1n file or [Create One](https://pumpbin.b1n.io/devs/start.html).
A b1n file contains one or more binary implant templates, along with some additional descriptive information.
We refer to a b1n file as a Plugin.

![](https://github.com/pumpbin/pumpbin/assets/120295547/2c94a40f-a370-4cef-a676-3a843e535edb)

## ✨ Features

- Powerful, simple, and comfortable UI
- Following the minimal principle to ensure maximum flexibility in usage
- Support two Plugin types: shellcode embedded in implants(Local) and hosted on remote servers(Remote)
- Support plugin development using any compiled language, such as C/C++, Rust, Zig, Go, etc
- Re-selecting the current plugin will generate a new random encryption password
- Filling with random data to ensure each generated implant is unique
- We have user manual, you no longer need to educate your users
- No dependencies, just PumpBin
- Support description, you can write down anything you want to remind users, which is important
- No network connection, eliminating any security concerns
- ... And I'm a pumpkin, I have magic🪄

## 🚀 Getting Started

Check the [PumpBin Documentation](https://pumpbin.b1n.io) for more information.

## ❔ Why

Modern cybersecurity teams are divided into offensive personnel and cybersecurity researchers,
with researchers responsible for producing digital weapons.
The teams typically deploy post-exploitationtools like Cobalt Strike, BRC4, or similar.
To evade security software, researchers usually write shellcode loaders, including evasion code to create the final implant.
This process generally follows two methods.

1. Offensive personnel provide the shellcode to researchers, who then directly produce the final implant.
   This method is highly inflexible as offensive personnel must contact researchers every time they need a final implant.

1. Researchers create a binary implant template and provide a final implant generation program.
   Offensive personnel use this program to inject shellcode into the binary implant template, producing the final implant.

The second method is the reason for the creation of PumpBin, a final implant generation program.
Cybersecurity researchers only need to follow PumpBin's guidelines to write implant templates and
distribute them along with PumpBin to offensive personnel. (There are very few guidelines as PumpBin is highly flexible.)
