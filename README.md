# hana_nagios_rust
 HANA Nagios Rust plugin
 ```
 Help :
  ./hana_nagios_rust
  usage: hana_nagios_rust HOSTNAME USERNAME PASSWORD SQLPORT MODE 
  
 Examples :
  ./hana_nagios_rust SAPHANAHOSTNAME SAPHANAUSER SAPHANAPASSWORD 30044 backup_data
  ./hana_nagios_rust [..] --mode backup_log
  ./hana_nagios_rust [..] --mode backup_data
  ./hana_nagios_rust [..] --mode db_log
  ./hana_nagios_rust [..] --mode db_data
  ./hana_nagios_rust [..] --mode cpu
  ./hana_nagios_rust [..] --mode memory
  ./hana_nagios_rust [..] --mode services
  ./hana_nagios_rust [..] --mode alert
```
=> see PREREQUISITES.TXT<br>
