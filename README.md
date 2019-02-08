# zoo
A command line password manager

## About
zoo is a password managing daemon built atop [ring.rs](https://github.com/briansmith/ring) intended for usage on servers 
where other GUI-like options are not available or impractical. 

## Installation
```
cargo install --git https://github.com/yasammez/zoo.git
```

## Usage

Once you have installed zoo, invoke it. It will ask you for a master password and then drop to the background. **Do not lose
that password!** It will then listen on a unix domain socket located at `$HOME/.local/share/zoo/socket`. You can connect to
it via openbsd netcat `nc -U ~/.local/share/zoo/socket` and issue commands directly. All changes made are instantaneously persisted, 
so be careful. Recognized commands are

* ?: get a list of all commands
* put &lt;key&gt; &lt;secret&gt;: create a new secret or override the value of an existing one
* del &lt;key&gt;: remove the associated secret from the vault
* get &lt;key&gt;: obtain the associated secret
* lst: get a list of all known keys
* off: shutdown the daemon

## Examples

Personally, I use zoo to provide my tmux statusline with a count of unread emails from various accounts. To this end, here are a couple
helper scripts, written in [fish](https://fishshell.com/) (if that doesn't tickle your fancy, they should be easy to convert to something
more POSIXy.

getpass.fish
```fish
function getpass --description 'get a password, fail if not found'
       echo -e "get $argv[1]\n" | nc -U ~/.local/share/zoo/socket | sed '/^val/!{q1}; s/^val //'
end
```

getmail.fish
```fish
function getmail --description 'get number of unread email in mailaccount'
        set pass (getpass $argv[2]); or return 1
        curl -s --url "imaps://$argv[1]:993/inbox;UID=1" --user "$argv[2]:$pass" -X 'STATUS INBOX (UNSEEN)' | sed 's/.*(UNSEEN \([0-9]*\))/\1/' | tr -d '\r\n'
end
```

updatemail.fish
```fish
function updatemail
        set result
        set accounts 'name:imap.example.com:user'
        for account in $accounts
                set args (string split : $account)
                set count (getmail $args[2] $args[3]); or return 1
                if [ "$count" != "0" ]
                        set result $result " $args[1] $count"
                end
        end
        echo $result > ~/.local/share/mailcount
end
```

I then include `updatemail` in my crontab to update the mailcount-file regularly and finally include
a section `#(cat ~/.local/share/mailcount)` in my tmuxline preset.

**Result**

![Screenshot](https://github.com/yasammez/zoo/raw/master/doc/example.png "Screenshot")
