#!/bin/sh

dfx canister call backend create

dfx canister call backend insert '(record {name = "person1"; age = 23})'

dfx canister call backend insert '(record {name = "person2"; age = 24})'

dfx canister call backend insert '(record {name = "person3"; age = 25})'

dfx canister call backend insert '(record {name = "person4"; age = 26})'

dfx canister call backend insert '(record {name = "person5"; age = 27})'

dfx canister call backend insert '(record {name = "person6"; age = 28})'

dfx canister call backend query '(record {limit = 3; offset = 2})'
