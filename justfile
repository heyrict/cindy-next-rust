watch:
	systemfd --no-pid -s http::7890 -- cargo watch -x run

keygen:
	ssh-keygen -t rsa -f key.pem -m PEM

signup:
	#!/usr/bin/env sh
	read -p 'Nickname: ' nickname
	read -p 'Username: ' username

	stty -echo
	read -p 'Password: ' password
	stty echo
	echo

	curl -iH 'Content-Type: application/json' http://${ENDPOINT}/signup -d '{"nickname": "'${nickname}'", "username": "'${username}'", "password": "'${password}'"}'

login:
	#!/usr/bin/env sh
	read -p 'Username: ' username

	stty -echo
	read -p 'Password: ' password
	stty echo
	echo

	curl -iH 'Content-Type: application/json' http://${ENDPOINT}/login -d '{"username": "'${username}'", "password": "'${password}'"}'

local_compile:
	docker build --rm -t cindy-next-rust --target builder .
