# Grab a snapshot of the `alpine` docker image from September 19, 2016.

FROM alpine@sha256:02eb5cfe4b721495135728ab4aea87418fd4edbfbf83612130a81191f0b2aae3

# Add a partial local mirror of the alpine latest repository, signed
# by the alpine developers.

ADD apks /apks
RUN echo "/apks/" > /etc/apk/repositories

# Install some utilities that we need.

RUN apk update
RUN apk add alpine-sdk
RUN apk add nano
RUN apk add xorriso
RUN apk add syslinux

# Add the 'builder' user (to the 'abuild' group)

RUN echo "builder:x:1000:100::/home/builder:/bin/bash" >> /etc/passwd
RUN mkdir /home/builder
RUN chown builder:abuild /home/builder
RUN sed -i.bak '/abuild/d' /etc/group
RUN echo "abuild:x:300:builder" >> /etc/group

# Add the APK keys to satisfy alpine's tooling. We do not
# rely on these keys for authentication or a source of trust;
# they are public.

RUN mkdir /home/builder/.abuild/
ADD iso.rsa /home/builder/.abuild/iso.rsa
RUN chmod 550 /home/builder/.abuild/iso.rsa
ADD iso.rsa.pub /home/builder/.abuild/iso.rsa.pub
RUN echo 'PACKAGER_PRIVKEY="/home/builder/.abuild/iso.rsa"' > /home/builder/.abuild/abuild.conf
RUN chown -R builder:abuild /home/builder/.abuild

RUN mkdir /root/.abuild/
ADD iso.rsa /root/.abuild/iso.rsa
RUN chmod 550 /root/.abuild/iso.rsa
ADD iso.rsa.pub /root/.abuild/iso.rsa.pub
RUN echo 'PACKAGER_PRIVKEY="/root/.abuild/iso.rsa"' > /root/.abuild/abuild.conf
RUN chown -R builder:abuild /root/.abuild

# Add the `mpc_compute` package.

ADD mpc_compute /home/builder/mpc_compute/
RUN chown -R builder:abuild /home/builder/mpc_compute/

# Add the `mpc_network` package

ADD mpc_network /home/builder/mpc_network/
RUN chown -R builder:abuild /home/builder/mpc_network/

# Run some setup as `builder`

# Build the MPC "compute" package that runs at boot.

USER builder
WORKDIR /home/builder/mpc_compute/

RUN abuild checksum
RUN abuild -r

# Build the MPC "network" package that runs at boot.

RUN rm /home/builder/packages/builder/x86_64/APKINDEX.tar.gz
WORKDIR /home/builder/mpc_network/

RUN abuild checksum
RUN abuild -r

# Move built packages into apks_custom.

RUN cp -r /home/builder/packages/builder /home/builder/apks_custom/

# Add the custom apks repository

USER root
RUN echo "/home/builder/apks_custom/" >> /etc/apk/repositories
ADD iso.rsa.pub /etc/apk/keys/iso.rsa.pub
RUN apk update
USER builder

# Grab the `alpine-iso` package.
WORKDIR /home/builder/
ADD alpine-iso /home/builder/alpine-iso/
WORKDIR /home/builder/alpine-iso/

# Grab the configurations for the ISOs
ADD alpine-compute.conf.mk /home/builder/alpine-iso/alpine-compute.conf.mk
ADD alpine-compute.packages /home/builder/alpine-iso/alpine-compute.packages
ADD alpine-network.conf.mk /home/builder/alpine-iso/alpine-network.conf.mk
ADD alpine-network.packages /home/builder/alpine-iso/alpine-network.packages

# Build the ISOs

USER root
RUN make BOOT_OPTS=pkgs=mpc_compute PROFILE=alpine-compute iso
RUN make BOOT_OPTS=pkgs=mpc_network PROFILE=alpine-network iso
RUN mkdir -p /home/builder/target/
