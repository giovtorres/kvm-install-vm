if [ -z "${VMNAME} ]
then
    TIMESTAMP=$(date '+%Y%m%d%H%M%S')
    VMNAME="batstestvm-${TIMESTAMP}"
fi
