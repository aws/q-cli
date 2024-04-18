import { Dispatch, SetStateAction, createContext } from "react";

type ModalProps = {
  modal: React.ReactNode | null;
  setModal: Dispatch<SetStateAction<React.ReactNode | null>>;
};

const modalObj: ModalProps = {
  modal: null,
  setModal: () => {},
};

const ModalContext = createContext(modalObj);

export default ModalContext;
