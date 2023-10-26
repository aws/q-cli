#!/bin/bash

TEAM_ID="94KV3E626L"

BUCKET_NAME="$1"
SIGNING_BUCKET="s3://$1"        # e.g. nscc-ec-signing-833388527378
SIGNING_REQUEST_QUEUE_NAME=$2   # e.g. nscc-signing-requests
NOTARIZING_SECRET_ID=$3         # e.g. nscc-notarizing-apple-id

set -eux

function signed_package_exists() {
    local name="$1"
    aws s3 ls "$SIGNING_BUCKET/signed/$name" &> /dev/null
    return $?
}

function post_request() {
    local message='{"type": "request", "command": "sign"}'
    local queue_url=$(aws sqs get-queue-url --queue-name "$SIGNING_REQUEST_QUEUE_NAME" | jq -r '.QueueUrl')
    aws sqs send-message --queue-url "$queue_url" --message-body "$message"
}

function build_signing_package() {
    local type=$1
    local full_file_path=$2
    local name=$3

    working_dir="./build-config/signing"
    starting_dir="$PWD"
   
    if [ "$type" = "dmg" ]
    then
        # Our dmg file names vary by platform, so this is templated in the manifest
        sed  "s/__NAME__/$name/g" < $working_dir/dmg/manifest.yaml.template > $working_dir/dmg/manifest.yaml
    fi

    cp -R "$full_file_path" "$working_dir/$type/artifact"
    rm -r "$full_file_path"
    gtar -czf "$working_dir/$type/artifact.gz" -C "$working_dir/$type/artifact" .
    cd "$working_dir/$type"
    gtar -czf "$starting_dir/package.tar.gz" manifest.yaml artifact.gz
    rm artifact.gz
    rm -r artifact/*
    cd "$starting_dir"
}

function sign_file() {
    local full_file_path=$1
    local name=$(basename "$full_file_path")
    local type="${name##*.}"

    echo "Signing $name"

    # Electric Company requires us to build up a tar file in an extremely specific format
    echo Packaging...
    build_signing_package "$type" "$full_file_path" "$name"

    # Upload package for signing to S3
    echo Uploading...
    aws s3 rm --recursive "$SIGNING_BUCKET/signed"
    aws s3 rm --recursive "$SIGNING_BUCKET/pre-signed"
    aws s3 cp package.tar.gz "$SIGNING_BUCKET/pre-signed/package.tar.gz"
    rm package.tar.gz

    # Tell the signing host there's something to sign
    echo Sending request...
    post_request

    # Loop until the signed package appears in the S3 bucket, for a maximum of 3 minutes
    max_duration=180
    end_time=$((SECONDS + max_duration))

    while [ $SECONDS -lt $end_time ]; do
        if signed_package_exists "$name"; then
            break
        else
            echo "No signed package yet. Waiting..."
            sleep 10
        fi
    done

    # Check if the loop ended due to the maximum duration being reached
    if [ $SECONDS -ge $end_time ]; then
        echo "Signed package did not appear, check signer logs at https://tiny.amazon.com/se9u6x33/IsenLink"
        exit 1
    fi

    echo "Signed!"

    # Put the signed file back in its original location
    echo Downloading...
    aws s3 cp "$SIGNING_BUCKET/signed/$name" "$name"
    tar -zxf "$name"
    cp -R Payload/* "$full_file_path"
    rm -rf Payload "$name"

    echo "Signing status of $full_file_path:"
    codesign -dv --deep --strict "$full_file_path"
}

function rebundle_dmg() {
    local dmg_path=$1
    local app_path=$2
    local mounting_loc="/Volumes/CodeWhisperer"

    # The dmg file that Tauri makes for us is quite nice, so let's just
    # crack it open and replace the .app file with the signed and 
    # notarized one

    echo "Rebundling $dmg_path..."

    rm -rf ~/temp.dmg

    # Convert the dmg to writable
    hdiutil convert "$dmg_path" -format UDRW -o ~/temp.dmg

    # Mount the dmg
    hdiutil attach ~/temp.dmg

    # Copy in the new app
    cp -R "$app_path" "$mounting_loc"

    # Unmount the dmg
    hdiutil detach "$mounting_loc"
    
    # Convert the dmg to zipped, read only - this is the only type that EC will accept!!
    rm -f "$dmg_path"
    hdiutil convert ~/temp.dmg -format UDZO -o "$dmg_path"
}

function notarize_file() {
    local original_file=$1
    local name=$(basename "$original_file")
    local type="${name##*.}"
    local file_to_notarize="$original_file"

    if [ "$type" = "app" ]
    then
        # We can submit dmg files as is, but we have to zip up app files in a specific way
        file_to_notarize="CodeWhisperer.zip"
        ditto -c -k --sequesterRsrc --keepParent "$original_file" "$file_to_notarize"
    fi

    xcrun notarytool submit "$file_to_notarize" --team-id "$TEAM_ID" --apple-id "$APPLE_ID" --password "$APPLE_ID_PASSWORD" --wait 
    xcrun stapler staple "$original_file"

    if [ "$type" = "app" ]
    then
         rm -rf "$file_to_notarize"
    fi
}

function get_secrets() {
    secret_string=$(aws secretsmanager get-secret-value --secret-id "$NOTARIZING_SECRET_ID" | jq -r '.SecretString')
    APPLE_ID=$(echo "$secret_string" | jq -r '.appleId')
    APPLE_ID_PASSWORD=$(echo "$secret_string" | jq -r '.appleIdPassword')
    if [ -z "$APPLE_ID" ] || [ -z "$APPLE_ID_PASSWORD" ]
    then
        return 1
    fi
}

if ! get_secrets; then
    echo "Problem obtaining secrets"
    exit 1
fi

BUILD_DIR="./build"
app=$(ls -d1 "$BUILD_DIR/CodeWhisperer.app")
dmg=$(ls -1 "$BUILD_DIR/CodeWhisperer.dmg")

if [ -z "$app" ] || [ -z "$dmg" ]; then
  echo "Build artifact(s) not present, bailing on signing"
  exit 1
fi

echo "Working on $app and $dmg ..."

# Sign the application
sign_file "$app"

# Notarize the application
notarize_file "$app"

# Rebundle the dmg file with the signed and notarized application
rebundle_dmg "$dmg" "$app"

# Sign the dmg
sign_file "$dmg"

# Notarize the dmg
notarize_file "$dmg"

echo "All good!!"

