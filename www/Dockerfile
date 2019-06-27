FROM nginx

COPY dist/* /usr/share/nginx/html/
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Insert wasm mime type
RUN sed -i 's/\}/    application\/wasm                      wasm;\n\}/' /etc/nginx/mime.types

# Let nginx use port given from env
CMD sed -i -e 's/$PORT/'"$PORT"'/g' /etc/nginx/conf.d/default.conf && nginx -g 'daemon off;'
