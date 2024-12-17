# Easy Deploy

**easy deploy** is a simple tool to help you deploy your applications to server by configure.

## Example

### 1. server config and deploy commands in one file.

config.yaml
```
server:
  host: "192.168.0.5"
  port: 8022
  user: "root"
  password: "xxxxxxxx"

files:
  - source: "buniness.jar"
    destination: "~/"
    commands:
      - "nohup java -jar ~/buniness.jar &"
```

deploy command:
```
deploy -c config.yaml
```

### 2. split server config and deploy commands into different files

deploy-commands.yaml
```
files:
  - source: "buniness.jar"
    destination: "~/"
    commands:
      - "nohup java -jar ~/buniness.jar &"
```

server-host.yaml
```
host: "192.168.0.5"
port: 8022
user: "root"
password: "xxxxxxxx"
```

deploy command:
```
deploy -c deploy-commands.yaml -s server-host.yaml
```
