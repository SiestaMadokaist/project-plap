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

class LoginUser extends BaseUsecase<IUser> {
    constructor(private repo: RegisterUserRepo, private props: TODO) {
        super();
    }
}

/** usecases/read-chapter.ts */
interface ReadChapterRepo {
    user(): UserRepository;
    chapter(): ChapterRepository;
    // need both user-repository to check bills/quota etc
    // need chapter to fetch the latest data of chapter.
}

class ReadChapter extends BaseUsecase<IChapter> {
    constructor(repo: ReadChapterRepo) {
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
    register = async (req: TODO): Promise<DTO<IUser>> => {
        const params = (req as any).body;
        /** pass the whole repository container */
        /** but register user only sees it as RegisterUserRepo which only has .user(), no access to .chapter() */
        const action = new RegisterUser(this.repoContainer, params);
        return action.exec();
    }

    login = async (req: TODO): Promise<DTO<IUser>> => {
        const params = (req as any).body;
        const action = new LoginUser(this.repoContainer, params);
        return action.exec();
    }

    profpic = async (req: TODO): Promise<Buffer> => {
        return Buffer.from('');
    }
}


/** middleware */
type NextFunction = Function;;
type Request = any;
type Response = any;
type Handler = any; // express.Handler;
const logMessage = (...args: unknown[]) => { };
export type AsyncHandler<T> = (req: Request, resp: Response, next: NextFunction) => Promise<T>
export function asyncJSONHandler<R>(handler: AsyncHandler<DTO<R>>): Handler {
    return async (req: Request, res: Response, next: NextFunction) => {
        logMessage(`${req.method}: ${req.path}`)
        try {
            const result = await handler(req, res, next);
            logMessage(`${req.method}: ${req.path} -> ${res.statusCode}`);
            res.json(await result.represent());
        } catch (err) {
            next(err);
        }
    };
}

export function asyncHandler<R>(handler: AsyncHandler<R>): Handler {
    return async (req: Request, res: Response, next: NextFunction) => {
        logMessage(`${req.method}: ${req.path}`)
        try {
            const result = await handler(req, res, next);
            logMessage(`${req.method}: ${req.path} -> ${res.statusCode}`);
            res.send(result);
        } catch (err) {
            next(err);
        }
    };
}

/** routes/user.rs */
const express = {} as any;
export const pdfRoute = (deps: RepoContainer) => {
    const controller = new UserController(deps);
    const router = express.Router();

    router.post('/queue', asyncJSONHandler(controller.register));
    router.post('/list', asyncJSONHandler(controller.login));
    router.get('/profile-picture/:id', asyncHandler(controller.profpic));
}