FROM openresty/openresty
COPY proxy.conf /etc/nginx/conf.d/default.conf
EXPOSE 80/tcp 443/tcp

#FROM ubuntu:20.04
#RUN apt-get -y update && apt-get -y install nginx
#COPY proxy.conf /etc/nginx/sites-available/default
#EXPOSE 80/tcp 443/tcp
#CMD ["/usr/sbin/nginx", "-g", "daemon off;"]

#Version 1.0
#FROM nginx:latest
#RUN rm /etc/nginx/conf.d/default.conf
#COPY proxy.conf /etc/nginx/conf.d/default.conf
#COPY index.html /usr/share/nginx/html
#COPY debug.sh /docker-entrypoint.d/
#COPY ssl /etc/ssl/private
