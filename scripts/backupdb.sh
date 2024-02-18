CURRENT_APPNAME=''
if [ -z "${APP_NAEM}" ]; then
    CURRENT_APPNAME='Conse'
else
    CURRENT_APPNAME=$APP_NAME
fi
sudo docker exec postgres pg_dump -U postgres $APP_NAME > ../$APP_NAME.sql