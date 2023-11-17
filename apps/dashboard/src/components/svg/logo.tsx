import { D, Svg } from "./icons";

export default function Lockup({ size = [132, 79] }: {size?: number | number[]}) {
  return(
    <Svg size={size} ratio={[132, 79]}>
      <D path="M58.6083 6.55908L51.0005 11.6309V14.9613C51.0005 15.6557 50.6799 16.3111 50.1319 16.7374L46.5005 19.5618V32.0945L50.1319 34.9189C50.6799 35.3452 51.0005 36.0006 51.0005 36.695V40.0254L58.6083 45.0972L63.8297 42.4865L64.5005 42.1511V38.5138C64.5005 38.3149 64.5795 38.1241 64.7202 37.9835L65.4702 37.2335C65.7631 36.9406 66.2379 36.9406 66.5308 37.2335L67.2808 37.9835C67.4215 38.1241 67.5005 38.3149 67.5005 38.5138V42.1511L68.1713 42.4865L73.3927 45.0972L81.0005 40.0254V36.695C81.0005 36.0006 81.3211 35.3452 81.8691 34.9189L85.5005 32.0945V19.5618L81.8691 16.7374C81.3211 16.3111 81.0005 15.6557 81.0005 14.9613V11.6309L73.3927 6.55908L68.1713 9.16978L67.5005 9.50519V13.1425C67.5005 13.3414 67.4215 13.5322 67.2808 13.6728L66.5308 14.4228C66.2379 14.7157 65.7631 14.7157 65.4702 14.4228L64.7202 13.6728C64.5795 13.5322 64.5005 13.3414 64.5005 13.1425V9.50519L63.8297 9.16978L58.6083 6.55908ZM57.3063 3.8215C57.9781 3.37362 58.8384 3.32005 59.5606 3.68115L65.1713 6.4865L66.0005 6.90109L66.8297 6.4865L72.4404 3.68115C73.1626 3.32005 74.0229 3.37362 74.6947 3.8215L82.9986 9.35742C83.6245 9.77471 84.0005 10.4772 84.0005 11.2295V14.5945L87.6319 17.4189C88.1799 17.8452 88.5005 18.5006 88.5005 19.195V32.4613C88.5005 33.1557 88.1799 33.8111 87.6319 34.2374L84.0005 37.0618V40.4268C84.0005 41.1791 83.6245 41.8816 82.9986 42.2989L74.6947 47.8348C74.0229 48.2827 73.1626 48.3362 72.4404 47.9751L66.8297 45.1698L66.0005 44.7552L65.1713 45.1698L59.5606 47.9751C58.8384 48.3362 57.9781 48.2827 57.3063 47.8348L49.0024 42.2989C48.3765 41.8816 48.0005 41.1791 48.0005 40.4268V37.0618L44.3691 34.2374C43.8211 33.8111 43.5005 33.1557 43.5005 32.4613V19.195C43.5005 18.5006 43.8211 17.8452 44.3691 17.4189L48.0005 14.5945V11.2295C48.0005 10.4772 48.3765 9.77471 49.0024 9.35742L57.3063 3.8215ZM67.1561 18.2674C67.2978 17.8782 67.7282 17.6775 68.1174 17.8191L69.527 18.3322C69.9162 18.4738 70.1169 18.9042 69.9752 19.2935L64.8449 33.3888C64.7032 33.7781 64.2729 33.9788 63.8836 33.8371L62.4741 33.3241C62.0849 33.1824 61.8842 32.752 62.0258 32.3628L67.1561 18.2674ZM58.4004 19.7008L53.6714 24.5988C53.1102 25.18 53.1102 26.1013 53.6714 26.6825L58.4004 31.5805C58.6881 31.8785 59.163 31.8868 59.4609 31.5991L60.54 30.5572C60.838 30.2695 60.8464 29.7947 60.5586 29.4967L56.8356 25.6407L60.5586 21.7846C60.8464 21.4866 60.838 21.0118 60.54 20.7241L59.4609 19.6822C59.163 19.3945 58.6881 19.4028 58.4004 19.7008ZM75.1654 25.6407L71.4423 21.7846C71.1546 21.4866 71.1629 21.0118 71.4609 20.7241L72.54 19.6822C72.838 19.3945 73.3128 19.4028 73.6005 19.7008L78.3296 24.5988C78.8908 25.18 78.8908 26.1013 78.3296 26.6825L73.6005 31.5805C73.3128 31.8785 72.838 31.8868 72.54 31.5991L71.4609 30.5572C71.1629 30.2695 71.1546 29.7947 71.4423 29.4967L75.1654 25.6407ZM73.7595 62.5281V72.7481H76.6795V62.5281H73.7595ZM73.9995 60.5281C74.3062 60.8081 74.7128 60.9481 75.2195 60.9481C75.7262 60.9481 76.1328 60.8081 76.4395 60.5281C76.7462 60.2481 76.8995 59.8681 76.8995 59.3881C76.8995 58.9081 76.7462 58.5281 76.4395 58.2481C76.1328 57.9681 75.7262 57.8281 75.2195 57.8281C74.7128 57.8281 74.3062 57.9681 73.9995 58.2481C73.6928 58.5281 73.5395 58.9081 73.5395 59.3881C73.5395 59.8681 73.6928 60.2481 73.9995 60.5281ZM29.945 71.8481L30.185 72.7481H32.725V57.9481H29.805V63.1281C29.0316 62.5281 28.1183 62.2281 27.065 62.2281C25.7716 62.2281 24.7316 62.7281 23.945 63.7281C23.1583 64.7148 22.765 66.0348 22.765 67.6881C22.765 68.7681 22.9383 69.7081 23.285 70.5081C23.645 71.3081 24.145 71.9215 24.785 72.3481C25.425 72.7615 26.165 72.9681 27.005 72.9681C28.1383 72.9681 29.1183 72.5948 29.945 71.8481ZM29.805 70.2081C29.2183 70.5815 28.605 70.7681 27.965 70.7681C27.205 70.7681 26.6383 70.5081 26.265 69.9881C25.905 69.4681 25.725 68.6748 25.725 67.6081C25.725 66.5415 25.8983 65.7481 26.245 65.2281C26.5916 64.6948 27.1183 64.4281 27.825 64.4281C28.5716 64.4281 29.2316 64.5948 29.805 64.9281V70.2081ZM11.3853 72.3881C10.3053 72.8015 9.09861 73.0081 7.76527 73.0081C5.51194 73.0081 3.80527 72.4081 2.64527 71.2081C1.48527 69.9948 0.905273 68.2215 0.905273 65.8881C0.905273 63.5815 1.51194 61.7948 2.72527 60.5281C3.93861 59.2615 5.64527 58.6281 7.84527 58.6281C9.01861 58.6281 10.1319 58.8148 11.1853 59.1881V61.5481C9.97194 61.2815 8.97861 61.1481 8.20527 61.1481C6.81861 61.1481 5.77861 61.5148 5.08527 62.2481C4.40527 62.9815 4.06527 64.1015 4.06527 65.6081V66.0681C4.06527 67.5615 4.39861 68.6748 5.06527 69.4081C5.73194 70.1281 6.75194 70.4881 8.12527 70.4881C8.89861 70.4881 9.98527 70.3348 11.3853 70.0281V72.3881ZM16.95 73.0481C15.3633 73.0481 14.1233 72.5748 13.23 71.6281C12.3366 70.6681 11.89 69.3348 11.89 67.6281C11.89 65.9348 12.3366 64.6148 13.23 63.6681C14.1233 62.7081 15.3633 62.2281 16.95 62.2281C18.5366 62.2281 19.7766 62.7081 20.67 63.6681C21.5633 64.6148 22.01 65.9348 22.01 67.6281C22.01 69.3348 21.5633 70.6681 20.67 71.6281C19.7766 72.5748 18.5366 73.0481 16.95 73.0481ZM16.95 70.8081C18.35 70.8081 19.05 69.7481 19.05 67.6281C19.05 65.5215 18.35 64.4681 16.95 64.4681C15.55 64.4681 14.85 65.5215 14.85 67.6281C14.85 69.7481 15.55 70.8081 16.95 70.8081ZM36.7111 68.3481C36.7644 69.2281 37.0311 69.8681 37.5111 70.2681C37.9911 70.6548 38.7444 70.8481 39.7711 70.8481C40.6644 70.8481 41.7044 70.6815 42.8911 70.3481V72.2881C42.4244 72.5281 41.8644 72.7148 41.2111 72.8481C40.5711 72.9815 39.9044 73.0481 39.2111 73.0481C37.5177 73.0481 36.2244 72.5881 35.3311 71.6681C34.4511 70.7481 34.0111 69.4015 34.0111 67.6281C34.0111 65.9215 34.4511 64.5948 35.3311 63.6481C36.2111 62.7015 37.4311 62.2281 38.9911 62.2281C40.3111 62.2281 41.3244 62.5948 42.0311 63.3281C42.7511 64.0481 43.1111 65.0815 43.1111 66.4281C43.1111 66.7215 43.0911 67.0548 43.0511 67.4281C43.0111 67.8015 42.9644 68.1081 42.9111 68.3481H36.7111ZM38.8911 64.2481C38.2377 64.2481 37.7177 64.4481 37.3311 64.8481C36.9577 65.2348 36.7444 65.8015 36.6911 66.5481H40.6111V66.2081C40.6111 64.9015 40.0377 64.2481 38.8911 64.2481ZM52.4514 62.9681L54.6514 72.7481H57.9314L61.7514 58.8881H58.5114L56.3514 69.4481L54.1114 59.2281H50.9514L48.7514 69.4881L46.5514 58.8881H43.2314L47.0714 72.7481H50.3314L52.4514 62.9681ZM68.9934 72.7481V66.0481C68.9934 65.5015 68.8734 65.1015 68.6334 64.8481C68.3934 64.5948 68.0267 64.4681 67.5334 64.4681C66.8001 64.4681 66.0734 64.6948 65.3534 65.1481V72.7481H62.4334V57.9481H65.3534V63.4481C66.5001 62.6348 67.7001 62.2281 68.9534 62.2281C69.9001 62.2281 70.6267 62.4881 71.1334 63.0081C71.6534 63.5148 71.9134 64.2415 71.9134 65.1881V72.7481H68.9934ZM82.8984 69.9281C82.8984 69.6615 82.8184 69.4548 82.6584 69.3081C82.5117 69.1615 82.2051 69.0015 81.7384 68.8281L80.1784 68.2081C79.3651 67.8881 78.7784 67.4948 78.4184 67.0281C78.0584 66.5615 77.8784 65.9681 77.8784 65.2481C77.8784 64.3415 78.2384 63.6148 78.9584 63.0681C79.6917 62.5081 80.6651 62.2281 81.8784 62.2281C82.9984 62.2281 84.0317 62.4481 84.9784 62.8881V64.8281C83.9651 64.5215 83.0184 64.3681 82.1384 64.3681C81.5917 64.3681 81.1917 64.4348 80.9384 64.5681C80.6851 64.7015 80.5584 64.9148 80.5584 65.2081C80.5584 65.4348 80.6317 65.6215 80.7784 65.7681C80.9384 65.9015 81.2651 66.0681 81.7584 66.2681L83.2784 66.8881C84.1051 67.2215 84.6917 67.6081 85.0384 68.0481C85.3984 68.4881 85.5784 69.0615 85.5784 69.7681C85.5784 70.7548 85.1984 71.5548 84.4384 72.1681C83.6917 72.7681 82.7051 73.0681 81.4784 73.0681C80.1051 73.0681 78.8917 72.7948 77.8384 72.2481V70.3081C79.1584 70.7215 80.3651 70.9281 81.4584 70.9281C82.4184 70.9281 82.8984 70.5948 82.8984 69.9281ZM86.4646 62.5281V76.8681H89.3847V71.9881C89.7313 72.2948 90.1513 72.5348 90.6446 72.7081C91.138 72.8815 91.658 72.9681 92.2047 72.9681C93.058 72.9681 93.8113 72.7348 94.4646 72.2681C95.118 71.7881 95.618 71.1348 95.9646 70.3081C96.3113 69.4815 96.4846 68.5415 96.4846 67.4881C96.4846 65.8748 96.1047 64.5948 95.3447 63.6481C94.598 62.7015 93.578 62.2281 92.2847 62.2281C91.698 62.2281 91.118 62.3415 90.5446 62.5681C89.9846 62.7948 89.5046 63.1015 89.1046 63.4881L88.8646 62.5281H86.4646ZM89.3847 64.9681C89.998 64.6081 90.6513 64.4281 91.3447 64.4281C92.1047 64.4281 92.658 64.6815 93.0046 65.1881C93.3513 65.6948 93.5247 66.5015 93.5247 67.6081C93.5247 68.7015 93.3447 69.5015 92.9846 70.0081C92.638 70.5148 92.0913 70.7681 91.3447 70.7681C90.6246 70.7681 89.9713 70.5881 89.3847 70.2281V64.9681ZM100.749 70.2681C101.229 70.6548 101.983 70.8481 103.009 70.8481C103.903 70.8481 104.943 70.6815 106.129 70.3481V72.2881C105.663 72.5281 105.103 72.7148 104.449 72.8481C103.809 72.9815 103.143 73.0481 102.449 73.0481C100.756 73.0481 99.4627 72.5881 98.5693 71.6681C97.6893 70.7481 97.2493 69.4015 97.2493 67.6281C97.2493 65.9215 97.6893 64.5948 98.5693 63.6481C99.4493 62.7015 100.669 62.2281 102.229 62.2281C103.549 62.2281 104.563 62.5948 105.269 63.3281C105.989 64.0481 106.349 65.0815 106.349 66.4281C106.349 66.7215 106.329 67.0548 106.289 67.4281C106.249 67.8015 106.203 68.1081 106.149 68.3481H99.9493C100.003 69.2281 100.269 69.8681 100.749 70.2681ZM102.129 64.2481C101.476 64.2481 100.956 64.4481 100.569 64.8481C100.196 65.2348 99.9827 65.8015 99.9293 66.5481H103.849V66.2081C103.849 64.9015 103.276 64.2481 102.129 64.2481ZM107.59 62.5281V72.7481H110.51V65.5281C111.256 65.2215 112.063 65.0681 112.93 65.0681C113.383 65.0681 113.796 65.1081 114.17 65.1881V62.4881C113.89 62.4481 113.623 62.4281 113.37 62.4281C112.836 62.4281 112.33 62.5548 111.85 62.8081C111.383 63.0481 110.883 63.4615 110.35 64.0481L109.99 62.5281H107.59ZM117.675 70.2681C118.155 70.6548 118.908 70.8481 119.935 70.8481C120.828 70.8481 121.868 70.6815 123.055 70.3481V72.2881C122.588 72.5281 122.028 72.7148 121.375 72.8481C120.735 72.9815 120.068 73.0481 119.375 73.0481C117.682 73.0481 116.388 72.5881 115.495 71.6681C114.615 70.7481 114.175 69.4015 114.175 67.6281C114.175 65.9215 114.615 64.5948 115.495 63.6481C116.375 62.7015 117.595 62.2281 119.155 62.2281C120.475 62.2281 121.488 62.5948 122.195 63.3281C122.915 64.0481 123.275 65.0815 123.275 66.4281C123.275 66.7215 123.255 67.0548 123.215 67.4281C123.175 67.8015 123.128 68.1081 123.075 68.3481H116.875C116.928 69.2281 117.195 69.8681 117.675 70.2681ZM119.055 64.2481C118.402 64.2481 117.882 64.4481 117.495 64.8481C117.122 65.2348 116.908 65.8015 116.855 66.5481H120.775V66.2081C120.775 64.9015 120.202 64.2481 119.055 64.2481ZM124.515 62.5281V72.7481H127.435V65.5281C128.182 65.2215 128.989 65.0681 129.855 65.0681C130.309 65.0681 130.722 65.1081 131.095 65.1881V62.4881C130.815 62.4481 130.549 62.4281 130.295 62.4281C129.762 62.4281 129.255 62.5548 128.775 62.8081C128.309 63.0481 127.809 63.4615 127.275 64.0481L126.915 62.5281H124.515Z" />
    </Svg>

  )
}