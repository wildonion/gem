

# Postgres Backup using Crontab


### step 0

```console
$ sudo chown -R root:root /home/conse/_backups && sudo chmod -R 777 /home/conse/_backups
```

### Step 1

```console
$ suso cp .pgpass /home/conse/
```

### Step 2

```console
$ sudo chown -R root:root /home/conse/.pgpass
```

### Step 3

```console
$ crontab backup
```
