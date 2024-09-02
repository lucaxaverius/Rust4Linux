#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <sys/ioctl.h>

// Define the IOCTL commands
#define IOCTL_MAGIC 's'
#define IOCTL_ADD_RULE _IOW(IOCTL_MAGIC, 1, struct IoctlArgument)
#define IOCTL_REMOVE_RULE _IOW(IOCTL_MAGIC, 2, struct IoctlArgument)

#define DEVICE_PATH "/dev/secrules"
#define RULE_SIZE 256
#define BUFFER_SIZE RULE_SIZE*1000

typedef unsigned int u32 ;

void ioctl_add_rule(u32 uid, const char *rule);
void ioctl_remove_rule(u32 uid, const char *rule);
void print_rules();

// Updated struct to match the kernel module
struct IoctlArgument {
    u32 uid;             // User ID
    char rule[RULE_SIZE]; // Rule string
};

void ioctl_add_rule(u32 uid, const char *rule) {
    int fd = open(DEVICE_PATH, O_WRONLY);
    if (fd < 0) {
        perror("Failed to open the device");
        return;
    }

    struct IoctlArgument arg;
    arg.uid = uid;
    strncpy(arg.rule, rule, RULE_SIZE);
    arg.rule[RULE_SIZE - 1] = '\0';  // Ensure null termination

    if (ioctl(fd, IOCTL_ADD_RULE, &arg) < 0) {
        perror("Failed to add rule via ioctl");
    }

    close(fd);
}

void ioctl_remove_rule(u32 uid, const char *rule) {
    int fd = open(DEVICE_PATH, O_WRONLY);
    if (fd < 0) {
        perror("Failed to open the device");
        return;
    }

    struct IoctlArgument arg;
    arg.uid = uid;
    strncpy(arg.rule, rule, RULE_SIZE);
    arg.rule[RULE_SIZE - 1] = '\0';  // Ensure null termination

    if (ioctl(fd, IOCTL_REMOVE_RULE, &arg) < 0) {
        perror("Failed to remove rule via ioctl");
    }

    close(fd);
}

void print_rules() {
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
    printf("%s", buffer);
    close(fd);
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Usage: %s <print|ioctl_add|ioctl_remove> [uid] [rule]\n", argv[0]);
        return -1;
    }

    u32 uid = (u32)atoi(argv[2]);

    if (strcmp(argv[1], "print") == 0) {
        print_rules();
    } else if (strcmp(argv[1], "ioctl_add") == 0) {
        if (argc != 4) {
            printf("Usage: %s ioctl_add <uid> <rule>\n", argv[0]);
            return -1;
        }
        ioctl_add_rule(uid, argv[3]);
    } else if (strcmp(argv[1], "ioctl_remove") == 0) {
        if (argc != 4) {
            printf("Usage: %s ioctl_remove <uid> <rule>\n", argv[0]);
            return -1;
        }
        ioctl_remove_rule(uid, argv[3]);
    } else {
        printf("Unknown command %s\n", argv[1]);
    }

    return 0;
}
