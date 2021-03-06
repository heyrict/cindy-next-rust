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
