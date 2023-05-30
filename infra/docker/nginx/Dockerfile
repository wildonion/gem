# Use the official Nginx image as the base image
FROM nginx:latest

# Remove the default Nginx configuration file
RUN rm /etc/nginx/conf.d/default.conf

# Copy the build folder of the React applications to the Nginx root directory
COPY build/conse /usr/share/nginx/html/conse
COPY build/panel /usr/share/nginx/html/conse-panel

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
#api.conse.app:
COPY cert-api.conse.app.pem /etc/nginx/cert-api.conse.app.pem
COPY key-api.conse.app.pem /etc/nginx/key-api.conse.app.pem

# COPY /etc/letsencrypt/live/deadcat.ir/fullchain.pem /etc/nginx/cert.pem
# COPY /etc/letsencrypt/live/deadcat.ir/privkey.pem /etc/nginx/key.pem
COPY options-ssl-nginx.conf /etc/nginx/options-ssl-nginx.conf
COPY ssl-dhparams.pem /etc/nginx/ssl-dhparams.pem

# Expose port 80 for HTTP traffic and port 443 for HTTPS traffic
EXPOSE 80
EXPOSE 443

# Configure Nginx to serve the React application over HTTPS
COPY nginx.conf /etc/nginx/nginx.conf
COPY conse.app.conf /etc/nginx/conse.conf
COPY panel.conse.app.conf /etc/nginx/conse.conf
COPY api.panel.conse.app.conf /etc/nginx/conse.conf
COPY api.conse.app.conf /etc/nginx/conse.conf


# Start Nginx in the foreground when the Docker container starts
CMD ["nginx", "-g", "daemon off;"]