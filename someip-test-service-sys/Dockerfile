FROM ubuntu:24.10

RUN apt-get update \
    && apt-get install -y wget unzip openjdk-8-jdk

RUN wget -q -O tmp.zip https://github.com/COVESA/capicxx-core-tools/releases/download/3.2.15/commonapi_core_generator.zip \
    && unzip tmp.zip -d /opt/capicxx-core-tools \
    && rm tmp.zip
RUN wget -q -O tmp.zip https://github.com/COVESA/capicxx-someip-tools/releases/download/3.2.15/commonapi_someip_generator.zip \
    && unzip tmp.zip -d /opt/capicxx-someip-tools \
    && rm tmp.zip
ENV PATH="$PATH:/opt/capicxx-core-tools:/opt/capicxx-someip-tools"

# Add host user and run under it to avoid generating files as a root.
ARG USER_NAME
ARG USER_UID
ARG USER_GID
RUN groupadd --gid $USER_GID $USER_NAME --force \
    && useradd --uid $USER_UID --gid $USER_GID -m $USER_NAME
USER $USER_NAME

# Explicitly specify shell to avoid build check warning.
# See: https://docs.docker.com/reference/build-checks/json-args-recommended/
SHELL ["/bin/bash", "-c"]
CMD commonapi-core-generator-linux-x86_64 -sk fidl/TestService.fidl -d cpp/service/gen \
    && commonapi-someip-generator-linux-x86_64 fidl/TestService.fdepl -d cpp/service/gen
