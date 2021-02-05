#define _GNU_SOURCE
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/mman.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>

int main() {
    int fd;
    char *file;
    fd = open("/tmp/test", O_RDWR | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC, 0600);
    //fcntl(fd, F_SETFD, FD_CLOEXEC);
    struct stat buf;
    fstat(fd, &buf);
    printf("st_dev: %d, st_ino: %d, st_size: %d, st_nlink: %d, st_mode:%d, st_blksize: %d, st_uid: %d, st_gid: %d, st_rdev: %d\n", buf.st_dev, buf.st_ino, buf.st_size, buf.st_nlink, buf.st_mode, buf.st_blksize, buf.st_uid, buf.st_gid, buf.st_rdev);
    unlink("/tmp/test");
    fstat(fd, &buf);
    printf("st_dev: %d, st_ino: %d, st_size: %d, st_nlink: %d, st_mode:%d, st_blksize: %d, st_uid: %d, st_gid: %d, st_rdev: %d\n", buf.st_dev, buf.st_ino, buf.st_size, buf.st_nlink, buf.st_mode, buf.st_blksize, buf.st_uid, buf.st_gid, buf.st_rdev);
    fallocate(fd, 0, 0, 48128000);
    file = mmap(0, 48128000, PROT_READ, MAP_SHARED, fd, 0);
    close(fd);

    fd = open("/tmp/test2", O_RDWR | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC, 0600);
    //fcntl(fd, F_SETFD, FD_CLOEXEC);
    fstat(fd, &buf);
    printf("st_dev: %d, st_ino: %d, st_size: %d, st_nlink: %d, st_mode:%d, st_blksize: %d, st_uid: %d, st_gid: %d, st_rdev: %d\n", buf.st_dev, buf.st_ino, buf.st_size, buf.st_nlink, buf.st_mode, buf.st_blksize, buf.st_uid, buf.st_gid, buf.st_rdev);
    unlink("/tmp/test2");
    fstat(fd, &buf);
    printf("st_dev: %d, st_ino: %d, st_size: %d, st_nlink: %d, st_mode:%d, st_blksize: %d, st_uid: %d, st_gid: %d, st_rdev: %d\n", buf.st_dev, buf.st_ino, buf.st_size, buf.st_nlink, buf.st_mode, buf.st_blksize, buf.st_uid, buf.st_gid, buf.st_rdev);
    fallocate(fd, 0, 0, 48128000);
    file = mmap(0, 48128000, PROT_READ, MAP_SHARED, fd, 0);
    close(fd);

    /*for (int i = 0; i < 48128000; ++i) {
        printf("%02x", file[i] & 0xFF);
        if ((i+1) % 1024 == 0)
          printf("\n");
    }*/
    printf("%lu\n", -4096);

    return 0;
}
