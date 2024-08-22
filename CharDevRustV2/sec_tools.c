#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>
#include <sys/ioctl.h>

// Define the IOCTL commands
#define IOCTL_MAGIC 's'
#define IOCTL_ADD_RULE _IOW(IOCTL_MAGIC, 1, struct IoctlCommand)
#define IOCTL_REMOVE_RULE _IOW(IOCTL_MAGIC, 2, struct IoctlCommand)

#define DEVICE_PATH "/dev/secrules"
#define BUFFER_SIZE 1024

struct IoctlCommand {
    char command[4];  // e.g., "add\0", "rmv\0" (4 bytes, including the null terminator)
    char rule[256];   // The rule string, up to 256 bytes
};

void ioctl_add_rule(const char *rule) {
    int fd = open(DEVICE_PATH, O_WRONLY);
    if (fd < 0) {
        perror("Failed to open the device");
        return;
    }

    struct IoctlCommand cmd;
    strncpy(cmd.command, "add", 4);  // Fill the command field with "add\0"
    strncpy(cmd.rule, rule, 256);    // Copy the rule into the rule field
    cmd.rule[255] = '\0';            // Ensure null termination if the rule is too long

    if (ioctl(fd, IOCTL_ADD_RULE, &cmd) < 0) {
        perror("Failed to add rule via ioctl");
    }

    close(fd);
}

void ioctl_remove_rule(const char *rule) {
    int fd = open(DEVICE_PATH, O_WRONLY);
    if (fd < 0) {
        perror("Failed to open the device");
        return;
    }

    struct IoctlCommand cmd;
    strncpy(cmd.command, "rmv", 4);  // Fill the command field with "rmv\0"
    strncpy(cmd.rule, rule, 256);    // Copy the rule into the rule field
    cmd.rule[255] = '\0';            // Ensure null termination if the rule is too long

    if (ioctl(fd, IOCTL_REMOVE_RULE, &cmd) < 0) {
        perror("Failed to remove rule via ioctl");
    }

    close(fd);
}

void add_rule(const char *rule) {
    int fd = open(DEVICE_PATH, O_WRONLY);
    if (fd < 0) {
        perror("Failed to open the device");
        return;
    }
    write(fd, rule, strlen(rule));
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
        printf("Usage: %s <add|print|ioctl_add|ioctl_remove> [rule]\n", argv[0]);
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
    } else if (strcmp(argv[1], "ioctl_add") == 0) {
        if (argc != 3) {
            printf("Usage: %s ioctl_add <rule>\n", argv[0]);
            return -1;
        }
        ioctl_add_rule(argv[2]);
    } else if (strcmp(argv[1], "ioctl_remove") == 0) {
        if (argc != 3) {
            printf("Usage: %s ioctl_remove <rule>\n", argv[0]);
            return -1;
        }
        ioctl_remove_rule(argv[2]);
    } else {
        printf("Unknown command %s\n", argv[1]);
    }

    return 0;
}
