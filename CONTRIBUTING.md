# Contributing to the project

Welcome to the project!  We're thrilled that you want to consider contributing to the project.  Everything helps, and we're delighted to onboard new contributors.

## How do I contribute?

There are a few ways you can contribute to the project:

1. **Reporting bugs**: If you find a bug, please open an issue on the project's GitHub repository.  We'll take a look at it and try to fix it as soon as possible.

2. **Suggesting features**: If you have an idea for a feature you'd like to see in the project, please open an issue on the project's GitHub repository.  We'll take a look at it and consider adding it to the project.

3. **Submitting pull requests**: If you'd like to contribute code to the project, please open a pull request on the project's GitHub repository.  We'll review your code and merge it if it meets our standards.

## What should I know before contributing?

Before contributing to the project, you should be familiar with the following:

1. **Git**: You should be familiar with Git and GitHub.  If you're not, you can learn more about Git [here](https://git-scm.com/).

2. **Markdown**: You should be familiar with Markdown, as the project's documentation is written in Markdown.  If you're not, you can learn more about Markdown [here](https://www.markdownguide.org/).

3. **Rust**: You should be familiar with Rust, as the project is written in Rust.  If you're not, you can learn more about Rust [here](https://www.rust-lang.org/).

## How do I get started?

To get started contributing to the project, follow these steps:

1. **Fork the project**: Click the "Fork" button on the project's GitHub repository to create your own copy of the project.

2. **Clone the project**: Clone your fork of the project to your local machine using the following command:

   ```sh
   git clone https://github.com/your-username/semrel.git
    ```
3. **Create a new branch**: Create a new branch for your changes using the following command:

   ```sh
   git checkout -b your-branch-name
   ```

4. **Make your changes**: Make your changes to the project.

5. **Commit your changes**: Make sure to use conventional commit messages when committing your changes.  You can learn more about conventional commits [here](https://www.conventionalcommits.org/).

6. **Push your changes**: Push your changes to your fork of the project using the following command:

   ```sh
   git push origin your-branch-name
   ```

7. **Open a pull request**: Open a pull request on the project's GitHub repository.  We'll review your changes and merge them if they meet our standards.

## Optional steps

### NeoVim setup

1. **Install the following plugins**:

### NeoVim setup

1. **Install the following plugins**:

- [rust.vim](https://github.com/rust-lang/rust.vim): Provides Rust file detection, syntax highlighting, formatting, and more.
- [coc.nvim](https://github.com/neoclide/coc.nvim): Intellisense engine for NeoVim.
- [coc-rust-analyzer](https://github.com/fannheyward/coc-rust-analyzer): Rust Analyzer extension for coc.nvim.
- [vim-toml](https://github.com/cespare/vim-toml): Syntax highlighting for TOML files.
- [vim-crates](https://github.com/saecki/crates.nvim): Provides information about Rust crates.

2. **Configure coc.nvim**: Add the following configuration to your `coc-settings.json` file:

```json
    {
    "rust-analyzer.server.path": "rust-analyzer",
    "rust-analyzer.cargo.runBuildScripts": true,
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.inlayHints.enable": true,
    "rust-analyzer.runnables.extraEnv": {
    "CARGO_PROFILE_DEV_DEBUG": true
    }
}
```

3. **Install and configure the plugins**: Add the following lines to your `init.vim` or `init.lua` file, depending on your configuration:

```vim
" init.vim
call plug#begin('~/.vim/plugged')
Plug 'rust-lang/rust.vim'
Plug 'neoclide/coc.nvim', {'branch': 'release'}
Plug 'fannheyward/coc-rust-analyzer'
Plug 'cespare/vim-toml'
Plug 'saecki/crates.nvim', {'do': 'UpdateRemotePlugins'}
call plug#end()
" Enable rust-analyzer
let g:coc_global_extensions = ['coc-rust-analyzer']
" Additional configuration for crates.nvim
lua << EOF
require('crates').setup()
EOF
```

```lua
lua
-- init.lua
require('packer').startup(function()
use 'rust-lang/rust.vim'
use {'neoclide/coc.nvim', branch = 'release'}
use 'fannheyward/coc-rust-analyzer'
use 'cespare/vim-toml'
use {'saecki/crates.nvim', run = 'UpdateRemotePlugins'}
end)
-- Enable rust-analyzer
vim.g.coc_global_extensions = {'coc-rust-analyzer'}
-- Additional configuration for crates.nvim
require('crates').setup()
```


### VSCode setup

1. **Install the following extensions**:

- [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer): This extension provides IDE-like features for Rust, such as code completion, syntax highlighting, and more.
- [Even-Better-TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml): This extension provides syntax highlighting for TOML files.
- [Crates](https://marketplace.visualstudio.com/items?itemName=serayuzgur.crates): This extension provides information about Rust crates.
- [LLDB](https://marketplace.visualstudio.com/items?itemName=Vadimcn.vscode-lldb): This extension provides debugging support for Rust using LLDB.

2. **Setup the Debugger**: Open `Command Palette` with `Ctrl+Shift+P` or `Cmd+Shift+P` on Mac and run `LLDB: Generate Launch Configuration from Cargo.toml`. This new file should be saved under `.vscode/launch.json`.  You may also want to enable debugging in the `.vscode/settings.json` file by adding the following lines:

```json
{
    "rust-analyzer.runnables.extraEnv": {
        "CARGO_PROFILE_DEV_DEBUG": true
    }
}
```