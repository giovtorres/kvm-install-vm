if [ -z "${VMNAME}" ]
then
    TIMESTAMP=$(date '+%Y%m%d%H%M%S')
    export VMNAME="batstestvm-${TIMESTAMP}"
fi
