ALTER TABLE servers ADD COLUMN server_path VARCHAR(255) DEFAULT '~/server';
ALTER TABLE servers ADD COLUMN start_command VARCHAR(500) DEFAULT './start.sh';
ALTER TABLE servers ADD COLUMN stop_command VARCHAR(500) DEFAULT 'pkill -f server';
