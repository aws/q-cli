#!/usr/bin/bash
set -eux

# This script is intended to run from cron in an Apollo environment with credentials vended by turtle
# It will not do anything useful in any other environment - don't try
CRED_ROOT="/apollo/env/CodewhispererEcSigner/var/credentials"

if [ -d "$CRED_ROOT" ]; then
    # The account we have credentials for is the account we should use
    AWS_ACCOUNT="$(ls -1 $CRED_ROOT)"
else
    echo "Environment is not correctly configured for this script to run"
    exit 1
fi

export AWS_DEFAULT_REGION="us-west-2"
export AWS_SHARED_CREDENTIALS_FILE="$CRED_ROOT/$AWS_ACCOUNT/fig-io-desktop-ec-signer-turtle-role/credentials"
QUEUE_URL="https://$AWS_DEFAULT_REGION.queue.amazonaws.com/$AWS_ACCOUNT/fig-io-desktop-signing-requests"
WAIT_TIME_SECONDS=20
LOG_GROUP_NAME="fig-io-desktop-build-signing-$AWS_ACCOUNT"
LOG_STREAM_NAME="fig-io-desktop-ec-signer"
LOCAL_LOG="/tmp/watch_for_signing_requests.log"
SIGNING_PROFILE_NAME="CodewhispererEcSigner"

BUCKET_NAME="fig-io-desktop-ec-signing-230592382359"
SOURCE_ARN="arn:aws:s3:::$BUCKET_NAME/pre-signed/package.tar.gz"
DESTINATION_ARN="arn:aws:s3:::$BUCKET_NAME/signed/package.tar.gz"
IAM_ROLE_ARN="arn:aws:iam::230592382359:role/codewhisperer-ec-signing-role"


export KRB5CCNAME=/apollo/env/${SIGNING_PROFILE_NAME}/var/krb5cc

# Redirect stdout and stderr to CloudWatch Logs and a local file
rm -f $LOCAL_LOG
exec > >(while IFS= read -r line; do t=$(date +%s%3N); echo $t $line >> $LOCAL_LOG; json_message=$(echo $line | jq -sRr '.' ); aws logs put-log-events --log-group-name "$LOG_GROUP_NAME" --log-stream-name "$LOG_STREAM_NAME" --log-events "timestamp=$t,message=\"$json_message\"" > /dev/null; done) 2>&1


# Receive messages from the SQS queue
echo "Waiting for requests"
messages=$(aws sqs receive-message --queue-url "$QUEUE_URL" --wait-time-seconds "$WAIT_TIME_SECONDS" --max-number-of-messages 1)

if [[ -n "$messages" ]]; then
    # Process the received messages

    requests="$(echo "$messages" | jq '.Messages')"
    request="$(echo "$requests" | jq -r '.[0]')"
    receipt_handle="$(echo "$request" | jq -r '.ReceiptHandle')"
    body="$(echo "$request" | jq -r '.Body')"
    type="$(echo "$body" | jq -r '.type')"

    if [ "$type" = "request" ]
    then
        echo "Request received, submitting"
        
        res=$(
            sudo curl -s -X POST \
              --negotiate -u : \
              -H "Content-Type: application/json" \
              -d "{ \"data\": { \"source\": { \"arn\": \"${SOURCE_ARN}\" }, \"destination\": { \"arn\": \"${DESTINATION_ARN}\" }, \"iam-role\": { \"arn\": \"${IAM_ROLE_ARN}\" }}}" \
              https://electric-company.integ.amazon.com/api/sign/app
        )

        task_id=$(echo "$res" | jq -r .data.task_id)
        if [[ -z "$task_id" || "$task_id" == "null" ]]; then
            echo "Signing request rejected:"
            echo "$res"
            exit 1
        fi

        f=0
        while true; do
            sres=$(
                sudo curl -s -X GET \
                    --negotiate -u : \
                    -H "Content-Type: application/json" \
                    "https://electric-company.integ.amazon.com/api/sign/$task_id/status"
            )

            result=$(echo "$sres" | jq -r '.data.status')

            case "$result" in
                "successful")
                    echo " success!"
                    exit 0
                    ;;
                "failed")
                    echo " failed (Task ID: ${task_id})"
                    exit 1
                    ;;
                "idle" | "in_progress")
                    if [[ $f -eq 1 ]]; then
                        echo -n "."
                    else
                        echo -n "Signing requested, waiting for results for task id ${task_id}: "
                        f=1
                    fi
                    sleep 2
                    ;;
                *)
                    echo "Unknown status for task ${task_id}: $result"
                    exit 2
                    ;;
            esac
        done


        # Delete the processed message from the queue
        aws sqs delete-message --queue-url "$QUEUE_URL" --receipt-handle "$receipt_handle"

        echo "Request removed from queue"
    else
        echo "Non-request message received, leaving on queue"
    fi
else
    echo "No requests this time"
fi

echo "Done."
