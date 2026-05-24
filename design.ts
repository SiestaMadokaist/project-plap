/** domain/types.ts */
type UserId = string;
type User = {};
type IUser = {};

type ChapterId = string;
type Chapter = {};
type IChapter = {};

type TODO = {};
const todo: TODO = {};

/** application/dto/base.ts */
abstract class DTO<T> {

    represent(): T {
        return todo as T;
    }
}

/** application/dto/response.ts */
class UserDTO extends DTO<IUser> {
    constructor(private u: User) {
        super();
    }

    override represent(): IUser {
        return this.u;
    }

}

/** repos/user.ts */
interface UserRepository {
    get(id: UserId): User
    register(email: string, password: string): User;
}

/** repos/chapter.ts */
interface ChapterRepository {
    get(id: ChapterId): Chapter;
}

/** repos/container.ts */
interface RepoContainer {
    user(): UserRepository;
    chapter(): ChapterRepository;
}

/** usecases/base.ts */
class BaseUsecase<T> {
    async exec(): Promise<DTO<T>> {
        return todo as DTO<T>; // todo
    }
}

/** usecases/register-user.ts */
interface RegisterUserRepo {
    user(): UserRepository;
    // this repo doesnt need ChapterRepository;
}

class RegisterUser extends BaseUsecase<IUser> {
    constructor(private repo: RegisterUserRepo, private props: TODO) {
        super();
    }
}

/** usecases/read-chapter.ts */
interface RegisterUserRepo {
    user(): UserRepository;
    chapter(): ChapterRepository;
    // need both user-repository to check bills/quota etc
    // need chapter to fetch the latest data of chapter.
}

class ReadChapter extends BaseUsecase<IChapter> {
    constructor(repo: RegisterUserRepo) {
        super();
    }
}

/** usecases/check-new-chapter.ts */
interface CheckNewChapterRepo {
    chapter(): ChapterRepository;
    // dont need user here.
}

class CheckNewChapter {
    constructor(repo: CheckNewChapterRepo) { }
}


/** controler/user.ts  */
class UserController {
    constructor(private repoContainer: RepoContainer) { };

    register = (params: TODO) => {
        /** pass the whole repository container */
        /** but register user only sees it as RegisterUserRepo which only has .user(), no access to .chapter() */
        const action = new RegisterUser(this.repoContainer, params);
        return action.exec();
    }
}