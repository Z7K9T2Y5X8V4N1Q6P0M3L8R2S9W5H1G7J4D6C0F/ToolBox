<p align="center">   English | <a href="./README_zh_CN.md">简体中文</a> </p>

<hr>

# ToolBox

<div align="center">
  <kbd>
    <img src="https://github.com/Z7K9T2Y5X8V4N1Q6P0M3L8R2S9W5H1G7J4D6C0F/ToolBox/blob/main/screenshot/1.png" />
    <img src="https://github.com/Z7K9T2Y5X8V4N1Q6P0M3L8R2S9W5H1G7J4D6C0F/ToolBox/blob/main/screenshot/2.png" />
  </kbd>
</div>


## Description

A Windows System Utilities Toolbox Demo, Built purely with Windows API, ensuring a minimal memory footprint.

### Features

- Settings Tab
  - Windows Update
    - Disable Windows Update
    - Pause Windows Update
    - Hide Windows Update

  - Explorer & UI
    - Disable Taskbar search advertisements
    - Enable Modern Explorer options
    - Disable Windows Spotlight
    - Disable Explorer modern search bar
    - Disable Explorer modern context menu
    - Disable Explorer Automatic Folder Type Discovery

  - Security & Performance
    - Remove Windows Defender services
    - Disable Core Isolation
    - Disable "Spectre" / "Meltdown" vulnerability patches
    - Disable SmartScreen

- Window Theme Styles Tab
  - Force target process to **"Basic"** style
  - Force target process to **"Classic"** style
  - Can be used in conjunction with [Advanced Appearance Settings](https://github.com/leetftw/SimpleClassicTheme/blob/master/SimpleClassicTheme/Resources/deskn.cpl)

- Menu Actions
  - Repair theme styles
  - Restore default classic theme style
  - Add extra classic theme styles
  - Global Basic style toggle
  - Global Classic style toggle


### Built with

- [Rust Toolchain](https://rust-lang.org)
- [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com)

## Getting started

### Prerequisites

**Operating System**: Supports Windows 10 or Windows 11 only.

**Build Environment**: Local compilation requires the installation of the Rust toolchain and Visual Studio C++ Build Tools.

### Install

This project works out of the box with zero external configuration required.

### Configure

```bash
git clone https://github.com/Z7K9T2Y5X8V4N1Q6P0M3L8R2S9W5H1G7J4D6C0F/ToolBox
cd ToolBox

cargo run --release
```

### Usage

Once compilation is complete, you can find the clean, standalone executable with zero external dependencies at the following path:
`.\target\release\TOOLBOX.exe`

## Back matter

### Legal disclaimer

Usage of this tool for attacking targets without prior mutual consent is illegal. It is the end user's responsibility to obey all applicable local, state, and federal laws. Developers assume no liability and are not responsible for any misuse or damage caused by this program.

### License

This project is licensed under the [MIT License](LICENSE).
