#include<stdio.h>
#include<stdlib.h>
#include"rdp.h"

Vec* readFile(char* path)
{
    FILE* fp;
    fp = fopen(path, "r");
    if (fp == NULL) {
        printf("Error, path: %s doesn't exist\n", path);
        exit(1);
    }
    fseek(fp, 0, SEEK_END);
    long fsize = ftell(fp);
    fseek(fp, 0, SEEK_SET);  /* same as rewind(f); */
    char* buff = malloc(fsize + 1);
    fread(buff, fsize, 1, fp);
    fclose(fp);
    buff[fsize] = 0;
    Vec* ret = new_vec(2);
    push_vec(ret, buff);
    push_vec(ret, &fsize);
    return ret;
}

void test_tokenizer() {
    Tokenizer* t = new_tokenizer("first second third fourth");
    // printf("here\n");
    printf("%s\n", get_next_token(t));
    printf("%s\n", get_next_token(t));
    printf("%s\n", get_next_token(t));
    printf("%s\n", get_next_token(t));
    
    free_tokenizer(t);
}

int main(int argc, char* argv[]) {
    test_tokenizer();
    // if (test_tokenizer()) {
        // Vec* contents = readFile("/Users/elocolburn/CompSci3/floralcc/text.txt");
        // Error success_code = program(get_vec(contents, 0), *(long*)get_vec(contents, 1));
        // printf("Success Code: %s", error_message(success_code));
    // } else printf("oops");
}