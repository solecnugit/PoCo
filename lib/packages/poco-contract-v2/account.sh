#!/bin/bash

create_account() {
    NEAR_MODE="env"
    source ./near-env.sh

    for num in $(seq 1 $1);
    do
        echo "Creating $num/$1 account"

        RANDOM_PREFIX=$(head -c 500 /dev/urandom | tr -dc 'a-z' | fold -w 5 | head -n 1)
        RANDOM_ACCOUNT="${RANDOM_PREFIX}.${NEAR_HELPER_ACCOUNT}"

        echo "Random account ${RANDOM_ACCOUNT}"
        echo $RANDOM_ACCOUNT >> ./accounts

        pnpm exec near create-account $RANDOM_ACCOUNT --masterAccount $NEAR_HELPER_ACCOUNT --initialBalance 100
    done
}

delete_account() {
    NEAR_MODE="env"
    source ./near-env.sh

    echo "Delete account $1"

    echo "y" | pnpm exec near delete $1 $NEAR_HELPER_ACCOUNT
}

if [ ! -n "$1" ]; then
    echo "$0 create [count]"
    echo "$0 delete [account]"
    echo "$0 clear"
    exit 1
elif [ "$1" = "create" ]; then
    if [ ! -n "$2" ]; then
        count=1
    else
        count=$2
    fi

    echo "Creating $count account(s)"

    create_account $count
elif [ "$1" = "delete" ]; then
    if [ $# == 2 ]; then
        delete_account $2
    else
        echo "$0 delete [account]"
        exit 1
    fi
elif [ "$1" = "clear" ]; then
    for line in $(cat ./accounts); do
        delete_account $line
    done

    cat /dev/null > ./accounts
fi
