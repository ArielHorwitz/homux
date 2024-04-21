Homux is a home directory multiplexer.

## How it works
A *source* directory is used as a source to be universally applied to the home directory of all hosts. In order to multiplex configurations across different hosts, the [matchpick](https://github.com/ArielHorwitz/matchpick) library is used to match different lines depending on which host is being applied to.

Let's look at an example configuration file, `~/.gitconfig`. Suppose our home computer is named "homestation", while our computer at work is named "workstation". We wish to configure our email address differently on each host, while keeping our name the same:
```
# https://git-scm.com/docs/git-config
[user]
    name = Tux Linux

    ~>>>
    email = "Default@example.com"
    ~>>> homestation
    email = "PersonalAccount@example.com"
    ~>>> workstation
    email = "WorkAccount@example.com"
    ~<<<
```

This allows a single directory managed by a single repository to be managed as the configuration source even across different hosts.
