# Use the official Nginx image as the base image
FROM nginx:latest

# Remove the default Nginx configuration file
RUN rm /etc/nginx/conf.d/default.conf

# Copy the build folder of the React application to the Nginx root directory
COPY build /usr/share/nginx/html

# Copy the SSL certificate and private key to the container
#conseapp:
COPY /etc/letsencrypt/live/conse.app/fullchain.pem /etc/nginx/cert-conse.pem
COPY /etc/letsencrypt/live/conse.app/privkey.pem /etc/nginx/key-conse.pem

# COPY /etc/letsencrypt/live/deadcat.ir/fullchain.pem /etc/nginx/cert.pem
# COPY /etc/letsencrypt/live/deadcat.ir/privkey.pem /etc/nginx/key.pem
COPY options-ssl-nginx.conf /etc/nginx/options-ssl-nginx.conf
COPY ssl-dhparams.pem /etc/nginx/ssl-dhparams.pem

# Expose port 80 for HTTP traffic and port 443 for HTTPS traffic
EXPOSE 80
EXPOSE 443

# Configure Nginx to serve the React application over HTTPS
COPY nginx.conf /etc/nginx/nginx.conf
COPY conse.conf /etc/nginx/conse.conf


# Start Nginx in the foreground when the Docker container starts
CMD ["nginx", "-g", "daemon off;"]