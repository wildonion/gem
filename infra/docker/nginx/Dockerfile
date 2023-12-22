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
#api.rendezvous.conse.app:
COPY cert-api.rendezvous.conse.app.pem /etc/nginx/cert-api.rendezvous.conse.app.pem
COPY key-api.rendezvous.conse.app.pem /etc/nginx/key-api.rendezvous.conse.app.pem
#adminer.conse.app:
COPY cert-adminer.conse.app.pem /etc/nginx/cert-adminer.conse.app.pem
COPY key-adminer.conse.app.pem /etc/nginx/key-adminer.conse.app.pem
#portainer.conse.app:
COPY cert-portainer.conse.app.pem /etc/nginx/cert-portainer.conse.app.pem
COPY key-portainer.conse.app.pem /etc/nginx/key-portainer.conse.app.pem
#grafana.conse.app:
COPY cert-grafana.conse.app.pem /etc/nginx/cert-grafana.conse.app.pem
COPY key-grafana.conse.app.pem /etc/nginx/key-grafana.conse.app.pem
#api.panel.stripewh.conse.app
COPY cert-api.panel.stripewh.conse.app.pem /etc/nginx/cert-api.panel.stripewh.conse.app.pem
COPY key-api.panel.stripewh.conse.app.pem /etc/nginx/key-api.panel.stripewh.conse.app.pem
#api.xbot.conse.app
COPY cert-api.xbot.conse.app.pem /etc/nginx/cert-api.xbot.conse.app.pem
COPY key-api.xbot.conse.app.pem /etc/nginx/key-api.xbot.conse.app.pem
#jenkins.conse.app:
COPY cert-jenkins.conse.app.pem /etc/nginx/cert-jenkins.conse.app.pem
COPY key-jenkins.conse.app.pem /etc/nginx/key-jenkins.conse.app.pem
#event.panel.conse.app:
COPY cert-event.panel.conse.app.pem /etc/nginx/cert-event.panel.conse.app.pem
COPY key-event.panel.conse.app.pem /etc/nginx/key-event.panel.conse.app.pem
#rpc.conse.app:
COPY cert-rpc.conse.app.pem /etc/nginx/cert-rpc.conse.app.pem
COPY key-rpc.conse.app.pem /etc/nginx/key-rpc.conse.app.pem

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
COPY api.rendezvous.conse.app.conf /etc/nginx/api.rendezvous.conse.app.conf
COPY adminer.conse.app.conf /etc/nginx/adminer.conse.app.conf
COPY portainer.conse.app.conf /etc/nginx/portainer.conse.app.conf
COPY grafana.conse.app.conf /etc/nginx/grafana.conse.app.conf
COPY jenkins.conse.app.conf /etc/nginx/jenkins.conse.app.conf
COPY event.panel.conse.app.conf /etc/nginx/event.panel.conse.app.conf
COPY api.panel.stripewh.conse.app.conf /etc/nginx/api.panel.stripewh.conse.app.conf
COPY api.xbot.conse.app.conf /etc/nginx/api.xbot.conse.app.conf
COPY rpc.conse.app.conf /etc/nginx/rpc.conse.app.conf

# Start Nginx in the foreground when the Docker container starts
CMD ["nginx", "-g", "daemon off;"]