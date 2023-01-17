

# Postgres Backup using Crontab


### step 0

```console
$ sudo chown -R root:root /home/ayoub/_backups && sudo chmod -R 777 /home/ayoub/_backups
```

### Step 1

```console
$ suso cp .pgpass /home/ayoub/
```

### Step 2

```console
$ sudo chown -R root:root /home/ayoub/.pgpass
```

### Step 3

```console
$ crontab backup
```
