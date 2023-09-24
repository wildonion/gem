# Use the official Nginx image as the base image
FROM nginx:latest

WORKDIR /etc/nginx

ADD . ./

WORKDIR /etc/nginx/infra/docker/nginx

# Remove the default Nginx configuration file
RUN rm /etc/nginx/conf.d/default.conf

# Copy the SSL certificate and private key to the container
#conse.app:
COPY cert-conse.app.pem /etc/nginx/cert-conse.app.pem
COPY key-conse.app.pem /etc/nginx/key-conse.app.pem
#panel.conse.app:
COPY cert-panel.conse.app.pem /etc/nginx/cert-panel.conse.app.pem
COPY key-panel.conse.app.pem /etc/nginx/key-panel.conse.app.pem
#api.panel.conse.app:
COPY cert-api.panel.conse.app.pem /etc/nginx/cert-api.panel.conse.app.pem
COPY key-api.panel.conse.app.pem /etc/nginx/key-api.panel.conse.app.pem
#api.mafia.conse.app:
COPY cert-api.mafia.conse.app.pem /etc/nginx/cert-api.mafia.conse.app.pem
COPY key-api.mafia.conse.app.pem /etc/nginx/key-api.mafia.conse.app.pem
#adminer.conse.app:
COPY cert-adminer.conse.app.pem /etc/nginx/cert-adminer.conse.app.pem
COPY key-adminer.conse.app.pem /etc/nginx/key-adminer.conse.app.pem
#portainer.conse.app:
COPY cert-portainer.conse.app.pem /etc/nginx/cert-portainer.conse.app.pem
COPY key-portainer.conse.app.pem /etc/nginx/key-portainer.conse.app.pem
#jenkins.conse.app:
COPY cert-jenkins.conse.app.pem /etc/nginx/cert-jenkins.conse.app.pem
COPY key-jenkins.conse.app.pem /etc/nginx/key-jenkins.conse.app.pem
#notif.panel.conse.app:
COPY cert-notif.panel.conse.app.pem /etc/nginx/cert-notif.panel.conse.app.pem
COPY key-notif.panel.conse.app.pem /etc/nginx/key-notif.panel.conse.app.pem

# COPY /etc/letsencrypt/live/deadcat.ir/fullchain.pem /etc/nginx/cert.pem
# COPY /etc/letsencrypt/live/deadcat.ir/privkey.pem /etc/nginx/key.pem
COPY options-ssl-nginx.conf /etc/nginx/options-ssl-nginx.conf
COPY ssl-dhparams.pem /etc/nginx/ssl-dhparams.pem

# Expose port 80 for HTTP traffic and port 443 for HTTPS traffic
EXPOSE 80
EXPOSE 443

COPY nginx.conf /etc/nginx/nginx.conf
COPY conse.app.conf /etc/nginx/conse.app.conf
COPY panel.conse.app.conf /etc/nginx/panel.conse.app.conf
COPY api.panel.conse.app.conf /etc/nginx/api.panel.conse.app.conf
COPY api.mafia.conse.app.conf /etc/nginx/api.mafia.conse.app.conf
COPY adminer.conse.app.conf /etc/nginx/adminer.conse.app.conf
COPY portainer.conse.app.conf /etc/nginx/portainer.conse.app.conf
COPY jenkins.conse.app.conf /etc/nginx/jenkins.conse.app.conf
COPY notif.panel.conse.app.conf /etc/nginx/notif.panel.conse.app.conf

# Start Nginx in the foreground when the Docker container starts
CMD ["nginx", "-g", "daemon off;"]