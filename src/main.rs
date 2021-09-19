use hdbconnect::{Connection, HdbResult, HdbReturnValue};
use std::env;
use std::process;

pub fn function_exit(status: &str) {
    if status == "OK" { process::exit(0); }
    if status == "WARNING" { process::exit(1); }
    if status == "CRITICAL"  { process::exit(2); }
    if status == "UNKNOWN" { process::exit(3); }
}

#[rustfmt::skip]
pub fn main() -> HdbResult<()> {
	
	    let hostname = env::args()
        .nth(1)
        .expect("Please provide HANA hostname in hana_nagios_rust hostname username password sqlport mode");
		let username = env::args()
        .nth(2)
        .expect("Please provide HANA username in hana_nagios_rust hostname username password sqlport mode");
		let password = env::args()
        .nth(3)
        .expect("Please provide HANA password in hana_nagios_rust hostname username password sqlport mode");
		let sqlport = env::args()
        .nth(4)
        .expect("Please provide HANA sql port in hana_nagios_rust hostname username password sqlport mode");
		let mode = env::args()
        .nth(5)
        .expect("Please provide mode, available modes: cpu, backup_data, backup_log, db_data, db_log, memory, services, alert in hana_nagios_rust hostname username password sqlport mode");
		
		
    // Get a connection
    let mut url = String::new();
    let mut result_status = String::new();
	url += "hdbsql://";
	url += &username;
	url += ":";
	url += &password;
	url += "@";
	url += &hostname;
	url += ":";
	url += 	&sqlport;
    let mut connection = Connection::new(url)?;
	
	match &mode[..] {
	"backup_data" => {
		let backupcnt: u32 = connection.query("SELECT count(*) FROM SYS.M_BACKUP_CATALOG where entry_type_name = 'complete data backup' and state_name = 'successful' and (sys_start_time between ADD_DAYS(current_timestamp, -3) and current_timestamp)")?.try_into()?;
		let mut lastbackupstr = String::new();	
		if backupcnt > 0 {
			result_status += "OK";
			let stmt = "SELECT top 1 sys_start_time FROM SYS.M_BACKUP_CATALOG where entry_type_name = 'complete data backup' and state_name='successful' order by entry_id desc";
			let lastbackup: Vec<String> = connection.query(stmt)?.try_into()?;
			lastbackupstr += "last successful ";	
			lastbackupstr += &lastbackup[0].to_string();
		} else {
			result_status += "CRITICAL";
			lastbackupstr += "No successful data backup";
		}
		println! ("{:?} - SAP HANA Data Backups: {:?}", result_status,lastbackupstr);
		function_exit(&result_status);
		},
	"backup_log" => {
		let backupcnt: u32 = connection.query("SELECT count(*) FROM SYS.M_BACKUP_CATALOG where entry_type_name = 'log backup' and state_name = 'successful' and (sys_start_time between ADD_SECONDS(current_timestamp, -10800) and current_timestamp);")?.try_into()?;
		let mut lastbackupstr = String::new();	
		if backupcnt > 0 {
			result_status += "OK";
			let stmt = "SELECT top 1 sys_start_time FROM SYS.M_BACKUP_CATALOG where entry_type_name = 'log backup' and state_name='successful' order by entry_id desc;";
			let lastbackup: Vec<String> = connection.query(stmt)?.try_into()?;
			lastbackupstr += "last successful ";	
			lastbackupstr += &lastbackup[0].to_string();
		} else {
			result_status += "CRITICAL";
			lastbackupstr += "No successful log backup";
		}
		println! ("{:?} - SAP HANA Log Backups: {:?}", result_status,lastbackupstr);
		function_exit(&result_status);
		},
	"memory" => {
		let mut memory = connection.statement("SELECT LPAD(TO_DECIMAL(ROUND(SUM(INSTANCE_TOTAL_MEMORY_USED_SIZE) OVER () / 1024 / 1024 / 1024), 10, 0), 9), LPAD(TO_DECIMAL(ROUND(SUM(INSTANCE_TOTAL_MEMORY_ALLOCATED_SIZE) OVER () / 1024 / 1024 / 1024), 10, 0), 9), LPAD(TO_DECIMAL(ROUND(SUM(ALLOCATION_LIMIT) OVER () / 1024 / 1024 / 1024), 10, 0), 9) FROM M_HOST_RESOURCE_UTILIZATION")?;
		memory.reverse();	
		for ret_val in memory {
			match ret_val {
				HdbReturnValue::ResultSet(rs) => {
					for row in rs {
						let mut row = row?;
						let mut row0: String = row.next_try_into()?;
						let mut row1: String = row.next_try_into()?;
						let mut row2: String = row.next_try_into()?;						
						row0.retain(|c| !c.is_whitespace());
						row1.retain(|c| !c.is_whitespace());
						row2.retain(|c| !c.is_whitespace());						
						let row0int: i32 = row0.parse().unwrap();
						let row1int: i32 = row1.parse().unwrap();
						let row2int: i32 = row2.parse().unwrap();						
						let result_1_80 = row1int * 80 / 100;
						let result_1_90 = row1int * 90 / 100;
						let result_percentage = 100 * row0int/row1int;
						if result_percentage <= 80 {result_status += "OK"}
						else if result_percentage >= 90 {result_status += "CRITICAL"}
						else if result_percentage > 80 && result_percentage < 90 {result_status += "WARNING"}
						println! ("{} - SAP HANA Used Memory ({}%) : {} GB Used / {} GB Allocated / {} GB Limit | mem={}MB;{};{};0;{}",result_status,result_percentage,row0int,row1int,row2int,row0int,result_1_80,result_1_90,row1int);
						function_exit(&result_status)
						}
					}
				HdbReturnValue::AffectedRows(affected_rows) => {
					println!("Got some affected rows counters: {:?}", affected_rows)
				}
				HdbReturnValue::Success => println!("Got success"),
				HdbReturnValue::OutputParameters(output_parameters) => {
					println!("Got output parameters: {:?}", output_parameters)
				}
				HdbReturnValue::XaTransactionIds(_) => println!("cannot happen"),
			}
		}
		},
	"services" => {
		let indexserver: Vec<String> = connection.query("SELECT ACTIVE_STATUS FROM SYS.M_SERVICES where IS_DATABASE_LOCAL='TRUE' AND SERVICE_NAME = 'indexserver';")?.try_into()?;
		if &indexserver[0].to_string() == "NO" {result_status += "CRITICAL";}
		if &indexserver[0].to_string() == "YES" {result_status += "OK";}
		println! ("{} - SAP HANA indexserver", result_status);
		function_exit(&result_status)
	},
	"cpu" => {	
		let mut cpu = connection.statement("SELECT STATUS,VALUE FROM SYS.M_SYSTEM_OVERVIEW WHERE SECTION='CPU' and NAME='CPU'")?;
		cpu.reverse();
		for ret_val in cpu {
			match ret_val {
			HdbReturnValue::ResultSet(rs) => {	
			for row in rs {	
						let mut row = row?;
						let f1: String = row.next_try_into()?;
						let f2: String = row.next_try_into()?;
						println! ("{} - SAP HANA {} CPU {}", f1, f1, f2);
						function_exit(&f1)
				}
			}
			HdbReturnValue::AffectedRows(affected_rows) => {
				println!("Got some affected rows counters: {:?}", affected_rows)
			}
			HdbReturnValue::Success => println!("Got success"),
			HdbReturnValue::OutputParameters(output_parameters) => {
				println!("Got output parameters: {:?}", output_parameters)
			}
			HdbReturnValue::XaTransactionIds(_) => println!("cannot happen"),
			}			
		}
	},
	"db_data" => {	
		let mut db_data = connection.statement("SELECT STATUS,VALUE FROM SYS.M_SYSTEM_OVERVIEW WHERE SECTION='Disk' and NAME='Data'")?;
		db_data.reverse();
		for ret_val in db_data {
			match ret_val {
			HdbReturnValue::ResultSet(rs) => {	
			for row in rs {	
						let mut row = row?;
						let f1: String = row.next_try_into()?;
						let f2: String = row.next_try_into()?;
						println! ("{} - SAP HANA {} Datafiles {}", f1, f1, f2);
						function_exit(&f1)
				}
			}
			HdbReturnValue::AffectedRows(affected_rows) => {
				println!("Got some affected rows counters: {:?}", affected_rows)
			}
			HdbReturnValue::Success => println!("Got success"),
			HdbReturnValue::OutputParameters(output_parameters) => {
				println!("Got output parameters: {:?}", output_parameters)
			}
			HdbReturnValue::XaTransactionIds(_) => println!("cannot happen"),
			}			
		}
	},
	"db_log" => {	
		let mut db_log = connection.statement("SELECT STATUS,VALUE FROM SYS.M_SYSTEM_OVERVIEW WHERE SECTION='Disk' and NAME='Log'")?;
		db_log.reverse();
		for ret_val in db_log {
			match ret_val {
			HdbReturnValue::ResultSet(rs) => {	
			for row in rs {	
						let mut row = row?;
						let f1: String = row.next_try_into()?;
						let f2: String = row.next_try_into()?;
						println! ("{} - SAP HANA {} Logfiles {}", f1, f1, f2);
						function_exit(&f1)
				}
			}
			HdbReturnValue::AffectedRows(affected_rows) => {
				println!("Got some affected rows counters: {:?}", affected_rows)
			}
			HdbReturnValue::Success => println!("Got success"),
			HdbReturnValue::OutputParameters(output_parameters) => {
				println!("Got output parameters: {:?}", output_parameters)
			}
			HdbReturnValue::XaTransactionIds(_) => println!("cannot happen"),
			}			
		}
	},
	"alerts" => {	
		let mut alerts = connection.statement("SELECT STATUS,VALUE FROM SYS.M_SYSTEM_OVERVIEW WHERE SECTION='Statistics' and NAME='Alerts'")?;
		alerts.reverse();
		for ret_val in alerts {
			match ret_val {
			HdbReturnValue::ResultSet(rs) => {	
			for row in rs {	
						let mut row = row?;
						let f1: String = row.next_try_into()?;
						let f2: String = row.next_try_into()?;
						if f1 != "OK" {
							let mut db_log = connection.statement("SELECT ALERT_RATING,ALERT_NAME,ALERT_DETAILS FROM _SYS_STATISTICS.STATISTICS_CURRENT_ALERTS WHERE ALERT_RATING >1")?;
							db_log.reverse();
							for ret_val in db_log {
								match ret_val {
									HdbReturnValue::ResultSet(rs) => {	
									for row in rs {	
										let mut row = row?;
										let f1a: String = row.next_try_into()?;
										let f2a: String = row.next_try_into()?;
										let f3a: String = row.next_try_into()?;
										println! ("{} - SAP HANA Alerts : {} | {}", f1a, f2a, f3a);
									}
								}
									HdbReturnValue::AffectedRows(affected_rows) => {
									println!("Got some affected rows counters: {:?}", affected_rows)
								}
									HdbReturnValue::Success => println!("Got success"),
									HdbReturnValue::OutputParameters(output_parameters) => {
									println!("Got output parameters: {:?}", output_parameters)
								}
									HdbReturnValue::XaTransactionIds(_) => println!("cannot happen"),
								}			
							}					
						} else {
							println! ("{} - SAP HANA Alerts {}", f1, f2);
						}
						function_exit(&f1)
				}
			}
			HdbReturnValue::AffectedRows(affected_rows) => {
				println!("Got some affected rows counters: {:?}", affected_rows)
			}
			HdbReturnValue::Success => println!("Got success"),
			HdbReturnValue::OutputParameters(output_parameters) => {
				println!("Got output parameters: {:?}", output_parameters)
			}
			HdbReturnValue::XaTransactionIds(_) => println!("cannot happen"),
			}			
		}
	},	
		_ => {println!("Mode {:?} not supported", mode);},
	}
	Ok(())
}
