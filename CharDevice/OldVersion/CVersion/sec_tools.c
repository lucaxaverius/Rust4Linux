#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>

#define DEVICE_PATH "/dev/secrules"
#define BUFFER_SIZE 1024

void add_rule(const char *rule)
{
    int fd = open(DEVICE_PATH, O_WRONLY);
    if (fd < 0) {
        perror("Failed to open the device");
        return;
    }
    write(fd, rule, strlen(rule));
    close(fd);
}

void print_rules()
{
    int fd = open(DEVICE_PATH, O_RDONLY);
    if (fd < 0) {
        perror("Failed to open the device");
        return;
    }
    char buffer[BUFFER_SIZE];
    int bytes_read = read(fd, buffer, BUFFER_SIZE - 1);
    if (bytes_read < 0) {
        perror("Failed to read from the device");
        close(fd);
        return;
    }
    buffer[bytes_read] = '\0';
    printf("%s\n", buffer);
    close(fd);
}

int main(int argc, char *argv[])
{
    if (argc < 2) {
        printf("Usage: %s <add|print> [rule]\n", argv[0]);
        return -1;
    }

    if (strcmp(argv[1], "add") == 0) {
        if (argc != 3) {
            printf("Usage: %s add <rule>\n", argv[0]);
            return -1;
        }
        add_rule(argv[2]);
    } else if (strcmp(argv[1], "print") == 0) {
        print_rules();
    } else {
        printf("Unknown command %s\n", argv[1]);
    }

    return 0;
}
