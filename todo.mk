Last login: Wed Oct 30 15:29:49 on ttys001
[oh-my-zsh] Would you like to update? [Y/n] y
Updating Oh My Zsh

master

Features:

 - c690f73                    Add `devcontainers` configuration (#12783)
 - 8da75e3 [buf]              Add completion plugin
 - a72a264 [chezmoi]          Add completion plugin (#12696)
 - f4423eb [cli]              Style plugin README in `omz plugin info`
 - 9bcafe1 [functions]        Add `takezip` (#12670)
 - 99e2c31 [git]              Add `git_previous_branch` function
 - 2a109d3 [git]              Add escape hatch to enable async prompt
 - d91944d [gnzh]             Add virtualenv prompt (#12666)
 - 6a10a4d [grep]             Exclude Python virtualenv from `grep` (#12685)
 - ba17328 [jonathan]         Add virtualenv support (#12705)
 - 4382288 [k9s]              Add completion plugin for `k9s` (#12691)
 - 1514145 [nvm]              Add `_omz_nvm_load` function
 - dae5a41 [opentofu]         Add `apply -auto-approve` alias (#12714)
 - 7ce26a8 [opentofu]         Add `destroy -auto-approve` alias (#12719)
 - 7bbebcd [rclone]           Create completion plugin (#12754)
 - 9ae1553 [scw]              Use official scw completion (#12755)
 - d2d5155 [ssh-agent]        Add keys regardless of filename (#12741)
 - c1679a1 [systemadmin]      Add `ping6` alias with count limit (#12697)
 - 62cf120 [terraform]        Add `destroy -auto-approve` alias (#12704)
 - d59f2fa [uv]               Add `uv` plugin (#12702)
 - a82f6c7 [wd]               Update to f0f47b71 (#12747)
 - e52598a [web-search]       Add `reddit` (#12664)

Bug fixes:

 - 9114853 [aussiegeek]       Quote color sequences
 - 767c927 [cli]              Add plugins with indentation in `omz plugin enable`
 - ec3cb12 [fastfile]         Use idiomatic expressions (#12708)
 - 61bacd9 [gem]              Regression with gem completion (#12735)
 - 3151c9c [git]              Re-add accidentally removed `gcn` (#12681)
 - 68d189c [last-working-dir] Save working directory more strictly (#11343)
 - 1b5af71 [pm2]              Update completion
 - 0987eee [poetry-env]       Only run `deactivate` if needed (#12701)
 - 2a2f8ec [ssh-agent]        Use termux prefix for tmp (#12695)
 - 0c8ce9d [theme-chooser]    Use `env` in shebang (#12720)
 - fa64758 [vagrant-prompt]   Make `vagrant_prompt_info` generic for any state (#12782)
 - eeb01c1 [websearch]        Allow multi-word parameters

Documentation:

 - 45516ca [jsontools]        Document requirements
 - 067558d [volta]            Fix typo (#12765)

Other changes:

 - 0a6f88b                    Style: Run prettier on main README
 - 865291c                    Feat (terraform): add `apply -auto-approve` alias (#12658)
 - 09a9467                    Revert "feat(ssh-agent): add keys regardless of filename (#12741)" (#12761)
 - 00b9b62 [bzr]              Refactor: Simplify and improve code (#12716)

You can see the changelog with `omz changelog`
         __                                     __   
  ____  / /_     ____ ___  __  __   ____  _____/ /_  
 / __ \/ __ \   / __ `__ \/ / / /  /_  / / ___/ __ \ 
/ /_/ / / / /  / / / / / / /_/ /    / /_(__  ) / / / 
\____/_/ /_/  /_/ /_/ /_/\__, /    /___/____/_/ /_/  
                        /____/                       

Hooray! Oh My Zsh has been updated!

To keep up with the latest news and updates, follow us on X: https://x.com/ohmyzsh
Want to get involved in the community? Join our Discord: https://discord.gg/ohmyzsh
Get your Oh My Zsh swag at: https://shop.planetargon.com/collections/oh-my-zsh
➜  ~ 
➜  ~ ssh home
^C
➜  ~ ssh anth@100.75.98.90
The authenticity of host '100.75.98.90 (100.75.98.90)' can't be established.
ED25519 key fingerprint is SHA256:7vmmuRcMNES2SwKkV+LyUrFIuTWpoLMBHkZrzukewHs.
This host key is known by the following other names/addresses:
    ~/.ssh/known_hosts:16: [192.168.0.62]:2255
Are you sure you want to continue connecting (yes/no/[fingerprint])? yes
Warning: Permanently added '100.75.98.90' (ED25519) to the list of known hosts.
# Tailscale SSH requires an additional check.
# To authenticate, visit: https://login.tailscale.com/a/60f58540101fb
# Authentication checked with Tailscale SSH.
# Time since last authentication: 1s
Last login: Thu Oct 24 10:29:28 from 192.168.0.136
➜  ~ usr                           
zsh: command not found: usr
➜  ~ whoami
anth
➜  ~ z zed     
➜  zed-debugger z zed
➜  zed git:(settings-ui) ✗ ls
assets              Dockerfile-collab.dockerignore   livekit.yaml
Cargo.lock          Dockerfile-cross                 nix
Cargo.toml          Dockerfile-cross.dockerignore    Procfile
clippy.toml         Dockerfile-distros               Procfile.postgrest
CODE_OF_CONDUCT.md  Dockerfile-distros.dockerignore  README.md
compose.yml         docs                             renovate.json
CONTRIBUTING.md     extensions                       rust-toolchain.toml
crates              flake.lock                       script
Cross.toml          flake.nix                        shell.nix
debug.plist         legal                            target
default.nix         LICENSE-AGPL                     tooling
docker-compose.sql  LICENSE-APACHE                   typos.toml
Dockerfile-collab   LICENSE-GPL
➜  zed git:(settings-ui) ✗ git st                                  
On branch settings-ui
Your branch is ahead of 'fork/settings-ui' by 2 commits.
  (use "git push" to publish your local commits)

Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
	modified:   crates/editor/src/editor_settings_controls.rs

no changes added to commit (use "git add" and/or "git commit -a")
➜  zed git:(settings-ui) ✗ cargo build
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.19s
➜  zed git:(settings-ui) ✗ git diff 
➜  zed git:(settings-ui) ✗ git add .
➜  zed git:(settings-ui) ✗ git ci -m "Make enum drop menu macro for settings ui"
[settings-ui 9efd0a6241] Make enum drop menu macro for settings ui
 1 file changed, 38 insertions(+), 35 deletions(-)
➜  zed git:(settings-ui) git push                                             
Enumerating objects: 28, done.
Counting objects: 100% (28/28), done.
Delta compression using up to 20 threads
Compressing objects: 100% (19/19), done.
Writing objects: 100% (19/19), 2.35 KiB | 2.35 MiB/s, done.
Total 19 (delta 16), reused 0 (delta 0), pack-reused 0 (from 0)
remote: Resolving deltas: 100% (16/16), completed with 9 local objects.
To github.com:Anthony-Eid/zed.git
   9d657e771e..9efd0a6241  settings-ui -> settings-ui
➜  zed git:(settings-ui) z ~        
➜  ~ z obsidian 
➜  obsidian ls
vault
➜  obsidian z vault   
➜  vault ls
Zed
➜  vault cd Zed                                       
➜  Zed ls
'Settings UI.md'
➜  Zed vim Settings\ UI.md                

- [ ] redact_private_values
- [ ] expand_excerpt_lines
- [ ] middle_click_paste
- [ ] double_click_in_multibuffer
- [ ] search_wrap
- [ ] search
- [ ] auto_signature_help
- [ ] show_signature_help_after_edits
- [ ] jupyter


### Pre Existing
- [x] Font
- [x] Font Size
- [x] Buffer Font Ligatures

~                                                                               
~                                                                               
~                                                                               
~                                                                               
~                                                                               
~                                                                               
~                                                                               
-- VISUAL BLOCK --                                  1x1       45,27         Bot

